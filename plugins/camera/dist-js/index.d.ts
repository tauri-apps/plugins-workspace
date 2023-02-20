export declare enum Source {
    Prompt = "PROMPT",
    Camera = "CAMERA",
    Photos = "PHOTOS"
}
export declare enum ResultType {
    Uri = "uri",
    Base64 = "base64",
    DataUrl = "dataUrl"
}
export declare enum CameraDirection {
    Rear = "REAR",
    Front = "FRONT"
}
export interface ImageOptions {
    quality?: number;
    allowEditing?: boolean;
    resultType?: ResultType;
    saveToGallery?: boolean;
    width?: number;
    height?: number;
    correctOrientation?: boolean;
    source?: Source;
    direction?: CameraDirection;
    presentationStyle?: 'fullscreen' | 'popover';
    promptLabelHeader?: string;
    promptLabelCancel?: string;
    promptLabelPhoto?: string;
    promptLabelPicture?: string;
}
export interface Image {
    data: string;
    assetUrl?: string;
    format: string;
    saved: boolean;
    exif: unknown;
}
export declare function getPhoto(options?: ImageOptions): Promise<Image>;
