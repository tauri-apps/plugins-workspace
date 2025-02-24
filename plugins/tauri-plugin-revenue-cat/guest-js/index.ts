import { invoke } from '@tauri-apps/api/core'

export async function ping(value: string): Promise<string | null> {
  return await invoke<{value?: string}>('plugin:revenue-cat|ping', {
    payload: {
      value,
    },
  }).then((r) => (r.value ? r.value : null));
}

export async function createCustomer(projectId: string, apiKey: string, customerId: string, email: string): Promise<string | null> {
  return await invoke<{value?: string}>('plugin:revenue-cat|createCustomer', {
    payload: {
      projectId,
      apiKey,
      customerId,
      email,
    },
  }).then((r) => (r.value ? r.value : null));
}

export async function fetchProjectOverviewData(projectId: string, apiKey: string): Promise<string | null> {
  return await invoke<{value?: string}>('plugin:revenue-cat|fetchProjectOverviewData', {
    payload: {
      projectId,
      apiKey,
    },
  }).then((r) => (r.value ? r.value : null));
}

export async function getCustomersData(projectId: string, apiKey: string): Promise<string | null> {
  return await invoke<{value?: string}>('plugin:revenue-cat|getCustomersData', {
    payload: {
      projectId,
      apiKey,
    },
  }).then((r) => (r.value ? r.value : null));
}
