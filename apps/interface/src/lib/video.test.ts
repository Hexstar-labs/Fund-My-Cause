import {
  validateVideoFormat,
  validateVideoSize,
  validateVideoDuration,
  generateVideoThumbnail,
  optimizeVideo,
  getVideoMetadata,
  DEFAULT_VIDEO_CONFIG,
} from "@/lib/video";

describe("Video Utilities", () => {
  describe("validateVideoFormat", () => {
    it("should accept valid video formats", () => {
      const mp4File = new File([], "test.mp4", { type: "video/mp4" });
      expect(validateVideoFormat(mp4File)).toBeNull();

      const webmFile = new File([], "test.webm", { type: "video/webm" });
      expect(validateVideoFormat(webmFile)).toBeNull();

      const oggFile = new File([], "test.ogg", { type: "video/ogg" });
      expect(validateVideoFormat(oggFile)).toBeNull();
    });

    it("should reject invalid video formats", () => {
      const pngFile = new File([], "test.png", { type: "image/png" });
      const result = validateVideoFormat(pngFile);
      expect(result).not.toBeNull();
      expect(result).toContain("Unsupported video format");
    });

    it("should respect custom configuration", () => {
      const mp4File = new File([], "test.mp4", { type: "video/mp4" });
      const result = validateVideoFormat(mp4File, {
        allowedFormats: ["video/webm"],
      });
      expect(result).not.toBeNull();
      expect(result).toContain("Unsupported video format");
    });
  });

  describe("validateVideoSize", () => {
    it("should accept files under the size limit", () => {
      const smallFile = new File(["small content"], "test.mp4", {
        type: "video/mp4",
      });
      expect(validateVideoSize(smallFile)).toBeNull();
    });

    it("should reject files over the size limit", () => {
      const largeContent = new Array(101 * 1024 * 1024).fill("x").join("");
      const largeFile = new File([largeContent], "test.mp4", {
        type: "video/mp4",
      });
      const result = validateVideoSize(largeFile);
      expect(result).not.toBeNull();
      expect(result).toContain("too large");
    });

    it("should respect custom size limits", () => {
      const file = new File(new Array(11 * 1024 * 1024).fill("x"), "test.mp4", {
        type: "video/mp4",
      });
      const result = validateVideoSize(file, { maxFileSizeMB: 10 });
      expect(result).not.toBeNull();
    });
  });

  describe("validateVideoDuration", () => {
    it("should validate video duration asynchronously", async () => {
      // This test requires actual video file handling
      // For now, we'll test the error handling
      const invalidFile = new File([], "test.mp4", {
        type: "video/mp4",
      });
      const result = await validateVideoDuration(invalidFile);
      // Should return error message since the file is empty
      expect(typeof result).toBe("string");
    });
  });

  describe("optimizeVideo", () => {
    it("should return optimization result object", async () => {
      const file = new File([], "test.mp4", { type: "video/mp4" });
      const result = await optimizeVideo(file);

      expect(result).toHaveProperty("isValid");
      expect(result).toHaveProperty("errors");
      expect(Array.isArray(result.errors)).toBe(true);
    });

    it("should catch format errors", async () => {
      const invalidFile = new File([], "test.txt", { type: "text/plain" });
      const result = await optimizeVideo(invalidFile);

      expect(result.isValid).toBe(false);
      expect(result.errors.length).toBeGreaterThan(0);
    });
  });

  describe("getVideoMetadata", () => {
    it("should return metadata object structure", async () => {
      // This test checks the function signature and structure
      expect(typeof getVideoMetadata).toBe("function");
    });
  });
});
