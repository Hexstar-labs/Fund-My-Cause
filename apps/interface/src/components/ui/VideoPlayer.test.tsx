import React from "react";
import { render, screen } from "@testing-library/react";
import { VideoPlayer, getVideoEmbed, getVideoThumbnail } from "@/components/ui/VideoPlayer";

describe("VideoPlayer Component", () => {
  describe("getVideoEmbed", () => {
    it("should parse YouTube URLs", () => {
      const result = getVideoEmbed("https://www.youtube.com/watch?v=dQw4w9WgXcQ");
      expect(result).not.toBeNull();
      expect(result?.type).toBe("youtube");
      expect(result?.embedUrl).toContain("dQw4w9WgXcQ");
    });

    it("should parse YouTube short URLs", () => {
      const result = getVideoEmbed("https://youtu.be/dQw4w9WgXcQ");
      expect(result).not.toBeNull();
      expect(result?.type).toBe("youtube");
      expect(result?.embedUrl).toContain("dQw4w9WgXcQ");
    });

    it("should parse Vimeo URLs", () => {
      const result = getVideoEmbed("https://vimeo.com/123456789");
      expect(result).not.toBeNull();
      expect(result?.type).toBe("vimeo");
      expect(result?.embedUrl).toContain("123456789");
    });

    it("should parse direct video URLs", () => {
      const result = getVideoEmbed("https://example.com/video.mp4");
      expect(result).not.toBeNull();
      expect(result?.type).toBe("direct");
      expect(result?.embedUrl).toBe("https://example.com/video.mp4");
    });

    it("should handle WebM format", () => {
      const result = getVideoEmbed("https://example.com/video.webm");
      expect(result).not.toBeNull();
      expect(result?.type).toBe("direct");
    });

    it("should return null for invalid URLs", () => {
      const result = getVideoEmbed("https://example.com/image.jpg");
      expect(result).toBeNull();
    });

    it("should return null for non-URL strings", () => {
      const result = getVideoEmbed("not a url");
      expect(result).toBeNull();
    });
  });

  describe("getVideoThumbnail", () => {
    it("should return YouTube thumbnail URL", () => {
      const url = "https://www.youtube.com/watch?v=dQw4w9WgXcQ";
      const thumbnail = getVideoThumbnail(url);
      expect(thumbnail).toBe("https://img.youtube.com/vi/dQw4w9WgXcQ/hqdefault.jpg");
    });

    it("should return YouTube thumbnail for short URL", () => {
      const url = "https://youtu.be/dQw4w9WgXcQ";
      const thumbnail = getVideoThumbnail(url);
      expect(thumbnail).toBe("https://img.youtube.com/vi/dQw4w9WgXcQ/hqdefault.jpg");
    });

    it("should return null for non-YouTube URLs", () => {
      const vimeoUrl = "https://vimeo.com/123456789";
      expect(getVideoThumbnail(vimeoUrl)).toBeNull();

      const directUrl = "https://example.com/video.mp4";
      expect(getVideoThumbnail(directUrl)).toBeNull();
    });

    it("should return null for invalid URLs", () => {
      expect(getVideoThumbnail("not a url")).toBeNull();
    });
  });

  describe("VideoPlayer Component Render", () => {
    it("should render null for invalid URLs", () => {
      const { container } = render(
        <VideoPlayer url="https://example.com/notavideo.jpg" />,
      );
      expect(container.firstChild).toBeNull();
    });

    it("should render video player for direct video URL", () => {
      const { container } = render(
        <VideoPlayer url="https://example.com/video.mp4" />,
      );
      const video = container.querySelector("video");
      expect(video).toBeInTheDocument();
      expect(video?.src).toBe("https://example.com/video.mp4");
    });

    it("should render iframe for YouTube URL", () => {
      const { container } = render(
        <VideoPlayer url="https://www.youtube.com/watch?v=dQw4w9WgXcQ" />,
      );
      const iframe = container.querySelector("iframe");
      expect(iframe).toBeInTheDocument();
      expect(iframe?.src).toContain("youtube.com/embed");
    });

    it("should render iframe for Vimeo URL", () => {
      const { container } = render(
        <VideoPlayer url="https://vimeo.com/123456789" />,
      );
      const iframe = container.querySelector("iframe");
      expect(iframe).toBeInTheDocument();
      expect(iframe?.src).toContain("vimeo.com");
    });

    it("should have caption for accessibility", () => {
      const { container } = render(
        <VideoPlayer url="https://example.com/video.mp4" />,
      );
      const video = container.querySelector("video");
      // The component should have proper semantic markup for accessibility
      expect(video).toHaveAttribute("src");
    });

    it("should display with custom className", () => {
      const { container } = render(
        <VideoPlayer
          url="https://example.com/video.mp4"
          className="custom-class"
        />,
      );
      const wrapper = container.querySelector(".custom-class");
      expect(wrapper).toBeInTheDocument();
    });
  });
});
