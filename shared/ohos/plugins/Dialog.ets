// Copyright 2019-2026 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT
import common from '@ohos.app.ability.common';
import picker from '@ohos.file.picker';
import fs from '@ohos.file.fs';
import util from '@ohos.util';
import window from '@ohos.window';
import promptAction from '@ohos.promptAction';
import { NativePlugin } from '../NativePlugin';
import { Files } from './Files';

interface MessageRequest {
  title: string; message: string; kind: string;
  okButtonLabel?: string; noButtonLabel?: string; cancelButtonLabel?: string;
}
interface MessageResponse { value: string; }
interface FileFilter { name: string; extensions: string[]; }
interface FileRequest {
  fileName?: string; filters: FileFilter[]; multiple: boolean;
  pickerMode?: string; fileAccessMode?: string;
}
interface OpenResponse { files: string[]; }
interface SaveResponse { file: string; }

export class Dialog implements NativePlugin {
  private context: common.UIAbilityContext;
  private files: Files;
  private active: boolean = true;
  constructor(context: common.UIAbilityContext, files: Files) { this.context = context; this.files = files; }

  private filters(request: FileRequest): string[] {
    return request.filters.map((filter: FileFilter): string => {
      const name = filter.name.split('|').join(' ');
      const extensions = filter.extensions.map((ext: string): string => ext === '*' ? '.*' : `.${ext}`).join(',');
      return `${name}|${extensions}`;
    });
  }

  async invoke(command: string, payload: string): Promise<string> {
    if (!this.active) throw new Error('Ability has been destroyed');
    if (command === 'showMessageDialog') {
      const request = JSON.parse(payload) as MessageRequest;
      const labels: string[] = [];
      if (request.okButtonLabel) labels.push(request.okButtonLabel);
      if (request.noButtonLabel) labels.push(request.noButtonLabel);
      if (request.cancelButtonLabel) labels.push(request.cancelButtonLabel);
      const buttons: promptAction.Button[] = labels.map((text: string, index: number): promptAction.Button => { return { text, color: index === 0 ? '#007DFF' : '#182431' }; });
      if (buttons.length === 0) throw new Error('A native dialog must have a button');
      const win = await window.getLastWindow(this.context);
      const result = await win.getUIContext().getPromptAction().showDialog({ title: request.title, message: request.message, buttons });
      if (!this.active) throw new Error('Ability has been destroyed');
      // Return the actual selected label, preserving Tauri's custom button contract.
      const response: MessageResponse = { value: result.index >= 0 && result.index < labels.length ? labels[result.index] : 'Cancel' };
      return JSON.stringify(response);
    }
    const request = JSON.parse(payload) as FileRequest;
    const view = new picker.DocumentViewPicker(this.context);
    if (command === 'showFilePicker') {
      let selected: string[];
      const media = request.pickerMode === 'media' || request.pickerMode === 'image' || request.pickerMode === 'video';
      if (media) {
        const options: picker.PhotoSelectOptions = {
          MIMEType: request.pickerMode === 'image' ? picker.PhotoViewMIMETypes.IMAGE_TYPE :
            request.pickerMode === 'video' ? picker.PhotoViewMIMETypes.VIDEO_TYPE : picker.PhotoViewMIMETypes.IMAGE_VIDEO_TYPE,
          maxSelectNumber: request.multiple ? 500 : 1
        };
        selected = (await new picker.PhotoViewPicker(this.context).select(options)).photoUris;
      } else {
        const options: picker.DocumentSelectOptions = { maxSelectNumber: request.multiple ? 500 : 1 };
        const filters = this.filters(request);
        if (filters.length > 0) options.fileSuffixFilters = filters;
        selected = await view.select(options);
      }
      if (!this.active) throw new Error('Ability has been destroyed');
      if (selected.length === 0) throw new Error('File selection cancelled');
      if (request.fileAccessMode === 'copy') {
        const directory = `${this.context.cacheDir}/tauri-picked-${util.generateRandomUUID()}`;
        await fs.mkdir(directory);
        const copied: string[] = [];
        for (const uri of selected) {
          const name = decodeURIComponent(uri.slice(uri.lastIndexOf('/') + 1)).split('/').join('_');
          const source = await fs.open(uri, fs.OpenMode.READ_ONLY);
          const target = `${directory}/${name}`;
          try { await fs.copyFile(source.fd, target); } finally { await fs.close(source); }
          copied.push(target);
        }
        selected = copied;
      } else {
        for (const uri of selected) this.files.grant(uri, !media);
      }
      const response: OpenResponse = { files: selected };
      return JSON.stringify(response);
    }
    if (command === 'saveFileDialog') {
      const options: picker.DocumentSaveOptions = { newFileNames: [request.fileName ?? 'Untitled'] };
      const filters = this.filters(request);
      if (filters.length > 0) options.fileSuffixChoices = filters;
      const selected = await view.save(options);
      if (!this.active) throw new Error('Ability has been destroyed');
      if (selected.length === 0) throw new Error('File save cancelled');
      this.files.grant(selected[0], true);
      const response: SaveResponse = { file: selected[0] };
      return JSON.stringify(response);
    }
    throw new Error(`Unsupported dialog command: ${command}`);
  }
  close(): void { this.active = false; }
}
