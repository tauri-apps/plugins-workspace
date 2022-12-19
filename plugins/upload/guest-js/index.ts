import { invoke } from "@tauri-apps/api/tauri";
import { appWindow } from "@tauri-apps/api/window";

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

enum UploadEvent {
  progress = "upload://progress",
  fileSize = "upload://file-size",
}

type EventId = `${UploadEvent}-${number}`;

type ProgressHandler = (data: ProgressHandlerData) => unknown;
type SizeHandler = (data: SizeHandlerData) => unknown;
const handlers: Map<EventId, ProgressHandler | SizeHandler> = new Map();
const listeningMap: Map<UploadEvent, boolean> = new Map();

const getIdForEvent = (event: UploadEvent, id: number): EventId =>
  `${event}-${id}`;

async function listenToEventIfNeeded(event: UploadEvent) {
  if (listeningMap.get(event)) {
    return;
  }
  return appWindow.listen<SizeHandlerData & ProgressHandlerData>(
    event,
    ({ payload }) => {
      const eventId = getIdForEvent(event, payload.id);
      const handler = handlers.get(eventId);
      if (typeof handler === "function") {
        handler(payload);
      }
    }
  );
}

export default async function upload(
  url: string,
  filePath: string,
  progressHandler?: ProgressHandler,
  fileSizeHandler?: SizeHandler,
  headers?: Record<string, string>
) {
  const ids = new Uint32Array(1);
  window.crypto.getRandomValues(ids);
  const id = ids[0];

  if (progressHandler) {
    const eventId = getIdForEvent(UploadEvent.progress, id);
    handlers.set(eventId, progressHandler);
  }

  if (fileSizeHandler) {
    const eventId = getIdForEvent(UploadEvent.fileSize, id);
    handlers.set(eventId, fileSizeHandler);
  }

  await listenToEventIfNeeded(UploadEvent.progress);
  await listenToEventIfNeeded(UploadEvent.fileSize);

  return await invoke<ResponseData>("plugin:upload|upload", {
    id,
    url,
    filePath,
    headers: headers ?? {},
  });
}
