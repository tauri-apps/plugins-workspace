<script>
  import { sendNotification } from '@tauri-apps/plugin-notification'
  export let onMessage

  let sound = ''

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
