interface BaseHandlerData {
    id: number;
}
interface ProgressHandlerData extends BaseHandlerData {
    progress: number;
    total: number;
}
interface SizeHandlerData extends BaseHandlerData {
    size: number;
}
interface ResponseData {
    text: string;
    status: number;
}
type ProgressHandler = (data: ProgressHandlerData) => unknown;
type SizeHandler = (data: SizeHandlerData) => unknown;
export default function upload(url: string, filePath: string, progressHandler?: ProgressHandler, fileSizeHandler?: SizeHandler, headers?: Record<string, string>): Promise<ResponseData>;
export {};
