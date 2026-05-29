import { uploadToPinata } from "@/lib/pinata";

/**
 * Video optimization configuration
 */
export interface VideoOptimizationConfig {
  maxFileSizeMB?: number;
  maxDurationSeconds?: number;
  allowedFormats?: string[];
}

/**
 * Default video optimization settings
 */
export const DEFAULT_VIDEO_CONFIG: VideoOptimizationConfig = {
  maxFileSizeMB: 100,
  maxDurationSeconds: 600, // 10 minutes
  allowedFormats: ["video/mp4", "video/webm", "video/ogg"],
};

/**
 * Validates if a file is a supported video format
 * @param file - The file to validate
 * @param config - Video optimization config
 * @returns Error message if invalid, null if valid
 */
export function validateVideoFormat(
  file: File,
  config: VideoOptimizationConfig = DEFAULT_VIDEO_CONFIG,
): string | null {
  const allowedFormats = config.allowedFormats || DEFAULT_VIDEO_CONFIG.allowedFormats || [];

  if (!allowedFormats.includes(file.type)) {
    return `Unsupported video format. Supported formats: ${allowedFormats.map((f) => f.split("/")[1]).join(", ")}`;
  }
  return null;
}

/**
 * Validates video file size
 * @param file - The file to validate
 * @param config - Video optimization config
 * @returns Error message if invalid, null if valid
 */
export function validateVideoSize(
  file: File,
  config: VideoOptimizationConfig = DEFAULT_VIDEO_CONFIG,
): string | null {
  const maxSize = (config.maxFileSizeMB || DEFAULT_VIDEO_CONFIG.maxFileSizeMB || 100) * 1024 * 1024;

  if (file.size > maxSize) {
    return `Video file is too large. Maximum size is ${config.maxFileSizeMB}MB.`;
  }
  return null;
}

/**
 * Gets video metadata (duration, dimensions)
 * @param file - The video file
 * @returns Promise resolving to video metadata
 */
export async function getVideoMetadata(
  file: File,
): Promise<{ duration: number; width: number; height: number }> {
  return new Promise((resolve, reject) => {
    const video = document.createElement("video");
    const objectUrl = URL.createObjectURL(file);

    video.onloadedmetadata = () => {
      URL.revokeObjectURL(objectUrl);
      resolve({
        duration: video.duration,
        width: video.videoWidth,
        height: video.videoHeight,
      });
    };

    video.onerror = () => {
      URL.revokeObjectURL(objectUrl);
      reject(new Error("Failed to load video metadata"));
    };

    video.src = objectUrl;
  });
}

/**
 * Validates video duration
 * @param file - The video file
 * @param config - Video optimization config
 * @returns Error message if invalid, null if valid
 */
export async function validateVideoDuration(
  file: File,
  config: VideoOptimizationConfig = DEFAULT_VIDEO_CONFIG,
): Promise<string | null> {
  try {
    const metadata = await getVideoMetadata(file);
    const maxDuration = config.maxDurationSeconds || DEFAULT_VIDEO_CONFIG.maxDurationSeconds || 600;

    if (metadata.duration > maxDuration) {
      const minutes = Math.round(maxDuration / 60);
      return `Video duration is too long. Maximum is ${minutes} minutes.`;
    }
    return null;
  } catch (error) {
    return "Could not read video duration";
  }
}

/**
 * Generates a thumbnail image from a video at a specific timestamp
 * @param file - The video file
 * @param timeOffset - Time offset in seconds (default: 0)
 * @returns Promise resolving to a Blob representing the thumbnail image
 */
export async function generateVideoThumbnail(
  file: File,
  timeOffset: number = 0,
): Promise<Blob> {
  return new Promise((resolve, reject) => {
    const video = document.createElement("video");
    const canvas = document.createElement("canvas");
    const ctx = canvas.getContext("2d");

    if (!ctx) {
      reject(new Error("Could not create canvas context"));
      return;
    }

    const objectUrl = URL.createObjectURL(file);

    video.onloadedmetadata = () => {
      canvas.width = video.videoWidth;
      canvas.height = video.videoHeight;
      video.currentTime = Math.min(timeOffset, video.duration - 0.1);
    };

    video.oncanplay = () => {
      ctx.drawImage(video, 0, 0);
      canvas.toBlob(
        (blob) => {
          URL.revokeObjectURL(objectUrl);
          if (blob) {
            resolve(blob);
          } else {
            reject(new Error("Failed to generate thumbnail"));
          }
        },
        "image/jpeg",
        0.8,
      );
    };

    video.onerror = () => {
      URL.revokeObjectURL(objectUrl);
      reject(new Error("Failed to load video"));
    };

    video.src = objectUrl;
  });
}

/**
 * Optimizes a video file (generates thumbnail and validates)
 * @param file - The video file
 * @param config - Video optimization config
 * @returns Promise resolving to optimization result with thumbnail
 */
export async function optimizeVideo(
  file: File,
  config: VideoOptimizationConfig = DEFAULT_VIDEO_CONFIG,
): Promise<{
  isValid: boolean;
  errors: string[];
  thumbnail?: Blob;
  metadata?: { duration: number; width: number; height: number };
}> {
  const errors: string[] = [];

  // Validate format
  const formatError = validateVideoFormat(file, config);
  if (formatError) errors.push(formatError);

  // Validate size
  const sizeError = validateVideoSize(file, config);
  if (sizeError) errors.push(sizeError);

  // Validate duration
  const durationError = await validateVideoDuration(file, config);
  if (durationError) errors.push(durationError);

  if (errors.length > 0) {
    return { isValid: false, errors };
  }

  try {
    const [thumbnail, metadata] = await Promise.all([
      generateVideoThumbnail(file, 1),
      getVideoMetadata(file),
    ]);

    return {
      isValid: true,
      errors: [],
      thumbnail,
      metadata,
    };
  } catch (error) {
    return {
      isValid: false,
      errors: [error instanceof Error ? error.message : "Optimization failed"],
    };
  }
}

/**
 * Uploads a video file to IPFS via Pinata
 * @param file - The video file to upload
 * @returns Promise resolving to IPFS URI
 */
export async function uploadVideoToPinata(file: File): Promise<string> {
  return uploadToPinata(file);
}

/**
 * Uploads a thumbnail image to IPFS via Pinata
 * @param thumbnailBlob - The thumbnail image blob
 * @param videoFileName - Original video file name (used to generate thumbnail filename)
 * @returns Promise resolving to IPFS URI
 */
export async function uploadThumbnailToPinata(
  thumbnailBlob: Blob,
  videoFileName: string,
): Promise<string> {
  const thumbnailFileName = `${videoFileName.replace(/\.[^/.]+$/, "")}-thumbnail.jpg`;
  const thumbnailFile = new File([thumbnailBlob], thumbnailFileName, { type: "image/jpeg" });
  return uploadToPinata(thumbnailFile);
}
