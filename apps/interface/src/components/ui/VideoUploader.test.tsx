import React from "react";
import { render, screen, fireEvent, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { VideoUploader } from "@/components/ui/VideoUploader";
import * as videoLib from "@/lib/video";
import * as pinatLib from "@/lib/pinata";

// Mock the video library
jest.mock("@/lib/video");
jest.mock("@/lib/pinata");

describe("VideoUploader Component", () => {
  beforeEach(() => {
    jest.clearAllMocks();
  });

  it("should render upload input", () => {
    render(<VideoUploader onUpload={jest.fn()} />);

    const uploadLabel = screen.getByText(/click to select a video file/i);
    expect(uploadLabel).toBeInTheDocument();
  });

  it("should display max file size and duration info", () => {
    render(<VideoUploader onUpload={jest.fn()} />);

    expect(screen.getByText(/max 100MB/i)).toBeInTheDocument();
    expect(screen.getByText(/10 min max/i)).toBeInTheDocument();
  });

  it("should handle file selection and upload", async () => {
    const mockOnUpload = jest.fn();
    const mockVideoUrl = "ipfs://QmTest123";

    (videoLib.optimizeVideo as jest.Mock).mockResolvedValue({
      isValid: true,
      errors: [],
      thumbnail: new Blob(["thumbnail"], { type: "image/jpeg" }),
    });

    (pinatLib.uploadVideoToPinata as jest.Mock).mockResolvedValue(mockVideoUrl);
    (pinatLib.uploadThumbnailToPinata as jest.Mock).mockResolvedValue(
      "ipfs://QmThumbnail123",
    );

    render(<VideoUploader onUpload={mockOnUpload} />);

    const input = screen.getByRole("button", {
      name: /click to select a video file/i,
    });

    const file = new File(["video content"], "test.mp4", {
      type: "video/mp4",
    });

    // Simulate file selection
    const fileInput = document.querySelector('input[type="file"]') as HTMLInputElement;
    expect(fileInput).toBeInTheDocument();

    fireEvent.change(fileInput, { target: { files: [file] } });

    await waitFor(() => {
      expect(videoLib.optimizeVideo).toHaveBeenCalledWith(file);
    });

    await waitFor(() => {
      expect(pinatLib.uploadVideoToPinata).toHaveBeenCalledWith(file);
    });

    await waitFor(() => {
      expect(mockOnUpload).toHaveBeenCalledWith(
        mockVideoUrl,
        "ipfs://QmThumbnail123",
      );
    });
  });

  it("should display errors from optimization", async () => {
    const mockOnError = jest.fn();
    const errorMessage = "Video format not supported";

    (videoLib.optimizeVideo as jest.Mock).mockResolvedValue({
      isValid: false,
      errors: [errorMessage],
    });

    render(
      <VideoUploader onUpload={jest.fn()} onError={mockOnError} />,
    );

    const file = new File(["content"], "test.txt", { type: "text/plain" });
    const fileInput = document.querySelector('input[type="file"]') as HTMLInputElement;

    fireEvent.change(fileInput, { target: { files: [file] } });

    await waitFor(() => {
      expect(screen.getByText(errorMessage)).toBeInTheDocument();
    });

    await waitFor(() => {
      expect(mockOnError).toHaveBeenCalledWith(errorMessage);
    });
  });

  it("should show success message after upload", async () => {
    const mockVideoUrl = "ipfs://QmTest123";

    (videoLib.optimizeVideo as jest.Mock).mockResolvedValue({
      isValid: true,
      errors: [],
      thumbnail: new Blob(["thumbnail"], { type: "image/jpeg" }),
    });

    (pinatLib.uploadVideoToPinata as jest.Mock).mockResolvedValue(mockVideoUrl);
    (pinatLib.uploadThumbnailToPinata as jest.Mock).mockResolvedValue(
      "ipfs://QmThumbnail123",
    );

    render(<VideoUploader onUpload={jest.fn()} />);

    const file = new File(["video content"], "test.mp4", {
      type: "video/mp4",
    });

    const fileInput = document.querySelector('input[type="file"]') as HTMLInputElement;
    fireEvent.change(fileInput, { target: { files: [file] } });

    await waitFor(() => {
      expect(screen.getByText(/Video uploaded successfully!/i)).toBeInTheDocument();
      expect(screen.getByText(mockVideoUrl)).toBeInTheDocument();
    });
  });

  it("should be disabled when disabled prop is true", () => {
    render(<VideoUploader onUpload={jest.fn()} disabled={true} />);

    const fileInput = document.querySelector('input[type="file"]') as HTMLInputElement;
    expect(fileInput).toBeDisabled();
  });
});
