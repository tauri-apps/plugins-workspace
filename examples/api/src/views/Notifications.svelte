<script>
  import { onDestroy, onMount } from 'svelte'
  import {
    onAction,
    registerActionTypes,
    sendNotification
  } from '@tauri-apps/plugin-notification'
  export let onMessage

  let sound = ''
  let actionListener

  onMount(async () => {
    await registerActionTypes([
      {
        id: 'api-example',
        actions: [
          { id: 'open', title: 'Open' },
          { id: 'later', title: 'Later' }
        ]
      }
    ]).catch(onMessage)
    // a click on the notification arrives as `tap`, a button as its id
    actionListener = await onAction((event) => {
      onMessage(
        `Action "${event.actionId}" on notification ${event.notification?.id}`
      )
    }).catch(onMessage)
  })

  onDestroy(() => actionListener?.unregister())

  function sendActionNotification() {
    sendNotification({
      id: Math.floor(Math.random() * 100000),
      title: 'Notification with actions',
      body: 'Click it or one of its buttons',
      actionTypeId: 'api-example'
    })
  }

  // send the notification directly
  // the backend is responsible for checking the permission
  function _sendNotification() {
    sendNotification({
      title: 'Notification title',
      body: 'This is the notification body',
      sound: sound || null
    })
  }

  // alternatively, check the permission ourselves
  function triggerNotification() {
    if (Notification.permission === 'default') {
      Notification.requestPermission()
        .then(function (response) {
          if (response === 'granted') {
            _sendNotification()
          } else {
            onMessage('Permission is ' + response)
          }
        })
        .catch(onMessage)
    } else if (Notification.permission === 'granted') {
      _sendNotification()
    } else {
      onMessage('Permission is denied')
    }
  }
</script>

<input
  class="input grow"
  placeholder="Notification sound..."
  bind:value={sound}
/>
<button class="btn" id="notification" on:click={triggerNotification}>
  Send test notification
</button>
<button class="btn" id="notification-actions" on:click={sendActionNotification}>
  Send notification with actions
</button>
