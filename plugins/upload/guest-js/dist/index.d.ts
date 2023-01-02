type ProgressHandler = (progress: number, total: number) => void;
declare function upload(url: string, filePath: string, progressHandler?: ProgressHandler, headers?: Map<string, string>): Promise<void>;
declare function download(url: string, filePath: string, progressHandler?: ProgressHandler, headers?: Map<string, string>): Promise<void>;
export default upload;
export { download, upload };
