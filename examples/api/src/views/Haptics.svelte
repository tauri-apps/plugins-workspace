<script>
  import {
    vibrate,
    impactFeedback,
    notificationFeedback,
    selectionFeedback
  } from '@tauri-apps/plugin-haptics'

  export let onMessage
</script>

<div>
  <button
    class="btn"
    on:click={() => vibrate(300).then(onMessage).catch(onMessage)}
    >vibrate short</button
  >
  <button
    class="btn"
    on:click={() => vibrate(1500).then(onMessage).catch(onMessage)}
    >vibrate long</button
  >
  <button
    class="btn"
    on:click={() => impactFeedback('medium').then(onMessage).catch(onMessage)}
    >impact medium</button
  >
  <button
    class="btn"
    on:click={() =>
      notificationFeedback('warning').then(onMessage).catch(onMessage)}
    >notification warning</button
  >
  <button
    class="btn"
    on:click={() => selectionFeedback().then(onMessage).catch(onMessage)}
    >selection</button
  >
</div>

<br />

<p>
  Depending on your device settings for haptic feedback some of the buttons may
  not work.
</p>
