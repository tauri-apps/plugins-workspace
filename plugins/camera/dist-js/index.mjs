import { invoke } from '@tauri-apps/api/tauri';

// Copyright 2019-2022 Tauri Programme within The Commons Conservancy
var Source;
(function (Source) {
    Source["Prompt"] = "PROMPT";
    Source["Camera"] = "CAMERA";
    Source["Photos"] = "PHOTOS";
})(Source || (Source = {}));
var ResultType;
(function (ResultType) {
    ResultType["Uri"] = "uri";
    ResultType["Base64"] = "base64";
    ResultType["DataUrl"] = "dataUrl";
})(ResultType || (ResultType = {}));
var CameraDirection;
(function (CameraDirection) {
    CameraDirection["Rear"] = "REAR";
    CameraDirection["Front"] = "FRONT";
})(CameraDirection || (CameraDirection = {}));
async function getPhoto(options) {
    return await invoke('plugin:camera|getPhoto', { ...options });
}

export { CameraDirection, ResultType, Source, getPhoto };
//# sourceMappingURL=index.mjs.map
