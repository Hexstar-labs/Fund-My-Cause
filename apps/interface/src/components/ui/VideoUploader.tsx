"use client";

import React, { useState } from "react";
import { Upload, AlertCircle, CheckCircle, Loader } from "lucide-react";
import {
  uploadVideoToPinata,
  uploadThumbnailToPinata,
  optimizeVideo,
  DEFAULT_VIDEO_CONFIG,
} from "@/lib/video";

interface VideoUploaderProps {
  onUpload: (videoUrl: string, thumbnailUrl?: string) => void;
  onError?: (error: string) => void;
  disabled?: boolean;
}

/**
 * VideoUploader Component
 * Handles video file uploads with optimization, validation, and thumbnail generation.
 */
export const VideoUploader: React.FC<VideoUploaderProps> = ({
  onUpload,
  onError,
  disabled = false,
}) => {
  const [uploading, setUploading] = useState(false);
  const [errors, setErrors] = useState<string[]>([]);
  const [success, setSuccess] = useState(false);
  const [uploadedVideoUrl, setUploadedVideoUrl] = useState<string | null>(null);

  const handleFileSelect = async (e: React.ChangeEvent<HTMLInputElement>) => {
    const file = e.target.files?.[0];
    if (!file) return;

    setErrors([]);
    setSuccess(false);
    setUploading(true);

    try {
      // Optimize the video (validate, generate thumbnail)
      const optimization = await optimizeVideo(file);

      if (!optimization.isValid) {
        const errorMessages = optimization.errors;
        setErrors(errorMessages);
        onError?.(errorMessages.join("; "));
        setUploading(false);
        return;
      }

      // Upload video
      const videoUrl = await uploadVideoToPinata(file);
      setUploadedVideoUrl(videoUrl);

      // Upload thumbnail if generated
      let thumbnailUrl: string | undefined;
      if (optimization.thumbnail) {
        thumbnailUrl = await uploadThumbnailToPinata(optimization.thumbnail, file.name);
      }

      setSuccess(true);
      onUpload(videoUrl, thumbnailUrl);

      // Reset success message after 5 seconds
      setTimeout(() => setSuccess(false), 5000);
    } catch (error) {
      const errorMessage = error instanceof Error ? error.message : "Upload failed";
      setErrors([errorMessage]);
      onError?.(errorMessage);
    } finally {
      setUploading(false);
    }
  };

  return (
    <div className="space-y-4">
      <div className="flex flex-col items-center justify-center w-full">
        <label
          htmlFor="video-upload"
          className={`flex flex-col items-center justify-center w-full h-32 border-2 border-dashed border-gray-700 rounded-xl cursor-pointer hover:border-indigo-500 transition ${
            uploading || disabled ? "opacity-50 cursor-not-allowed" : ""
          }`}
        >
          <div className="flex flex-col items-center justify-center pt-5 pb-6">
            <Upload size={24} className="text-gray-400 mb-2" />
            <p className="text-sm text-gray-400">
              {uploading ? "Uploading video…" : "Click to select a video file"}
            </p>
            <p className="text-xs text-gray-500 mt-1">
              Max {DEFAULT_VIDEO_CONFIG.maxFileSizeMB}MB, {DEFAULT_VIDEO_CONFIG.maxDurationSeconds! / 60} min max
            </p>
          </div>
          <input
            id="video-upload"
            type="file"
            accept={DEFAULT_VIDEO_CONFIG.allowedFormats?.join(",")}
            onChange={handleFileSelect}
            disabled={uploading || disabled}
            className="hidden"
          />
        </label>
      </div>

      {/* Errors */}
      {errors.length > 0 && (
        <div className="bg-red-50 dark:bg-red-900/30 border border-red-200 dark:border-red-800 rounded-lg p-3">
          <div className="flex gap-2">
            <AlertCircle size={18} className="text-red-600 dark:text-red-400 flex-shrink-0 mt-0.5" />
            <div>
              {errors.map((error, idx) => (
                <p key={idx} className="text-sm text-red-700 dark:text-red-300">
                  {error}
                </p>
              ))}
            </div>
          </div>
        </div>
      )}

      {/* Success */}
      {success && uploadedVideoUrl && (
        <div className="bg-green-50 dark:bg-green-900/30 border border-green-200 dark:border-green-800 rounded-lg p-3">
          <div className="flex gap-2">
            <CheckCircle size={18} className="text-green-600 dark:text-green-400 flex-shrink-0 mt-0.5" />
            <div>
              <p className="text-sm text-green-700 dark:text-green-300 font-medium mb-1">
                Video uploaded successfully!
              </p>
              <p className="text-xs text-green-600 dark:text-green-400 break-all">{uploadedVideoUrl}</p>
            </div>
          </div>
        </div>
      )}

      {/* Uploading indicator */}
      {uploading && (
        <div className="flex items-center gap-2 text-indigo-600 dark:text-indigo-400">
          <Loader size={16} className="animate-spin" />
          <span className="text-sm">Processing video…</span>
        </div>
      )}
    </div>
  );
};

export default VideoUploader;
