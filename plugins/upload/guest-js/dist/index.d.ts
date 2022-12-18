type ProgressHandler = (progress: number, total: number) => void;
export default function upload(url: string, filePath: string, progressHandler?: ProgressHandler, headers?: Map<string, string>): Promise<void>;
export {};
