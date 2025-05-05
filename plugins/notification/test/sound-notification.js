// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

/**
 * This file demonstrates how to use sound with notifications in Tauri v2.
 * 
 * On macOS: 
 * - Sound file should be in your application bundle
 * - Use the sound name without extension
 * - Or use system sounds like "Ping", "Basso", "Blow", "Bottle", "Frog", "Funk", "Glass", etc.
 * 
 * On Linux:
 * - Use a file path to a sound file (e.g., .wav file)
 * - Or use XDG theme sounds by name (e.g., "message-new-instant")
 * 
 * On Windows:
 * - Use a file path to a sound file (e.g., .wav file)
 */

import { sendNotification } from '@tauri-apps/api/notification';

// Example with system sound (macOS)
async function showNotificationWithSystemSound() {
  await sendNotification({
    title: 'Notification with System Sound',
    body: 'This notification uses a system sound on macOS.',
    sound: 'Ping' // macOS system sound
  });
}

// Example with custom sound file
async function showNotificationWithCustomSound() {
  await sendNotification({
    title: 'Notification with Custom Sound',
    body: 'This notification uses a custom sound file.',
    sound: 'notification.wav' // path to your sound file
  });
}

// Example with different sounds based on platform
async function showNotificationWithPlatformSpecificSound() {
  const platform = await import('@tauri-apps/api/os').then(os => os.platform());
  
  let soundPath;
  if (platform === 'darwin') {
    soundPath = 'Blow'; // macOS system sound
  } else if (platform === 'linux') {
    soundPath = 'message-new-instant'; // XDG theme sound
  } else {
    soundPath = 'notification.wav'; // Custom sound file for Windows
  }
  
  await sendNotification({
    title: 'Platform-specific Sound',
    body: `This notification uses platform-specific sound settings for ${platform}.`,
    sound: soundPath
  });
}

// Export functions to be called from HTML
window.showNotificationWithSystemSound = showNotificationWithSystemSound;
window.showNotificationWithCustomSound = showNotificationWithCustomSound;
window.showNotificationWithPlatformSpecificSound = showNotificationWithPlatformSpecificSound; 