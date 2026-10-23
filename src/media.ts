export const VIDEO_EXTS = ["mp4", "mov", "m4v", "webm"];
export const MEDIA_EXTS = ["jpg", "jpeg", "png", "gif", "webp", "heic", "heif", "bmp", "tiff", ...VIDEO_EXTS];

const ext = (path: string) => path.slice(path.lastIndexOf(".") + 1).toLowerCase();

export const isVideo = (path: string) => VIDEO_EXTS.includes(ext(path));

export const fileName = (path: string) => path.slice(path.lastIndexOf("/") + 1);
