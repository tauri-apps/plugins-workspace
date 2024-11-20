<script>
  import {
    scan,
    checkPermissions,
    requestPermissions,
    Format,
    cancel
  } from '@tauri-apps/plugin-barcode-scanner'

  export let onMessage

  let scanning = false
  let windowed = true
  let formats = [Format.QRCode]
  const supportedFormats = [Format.QRCode, Format.EAN13]

  async function startScan() {
    let permission = await checkPermissions()
    if (permission === 'prompt') {
      permission = await requestPermissions()
    }
    if (permission === 'granted') {
      scanning = true
      scan({ windowed, formats })
        .then((res) => {
          scanning = false
          onMessage(res)
        })
        .catch((error) => {
          scanning = false
          onMessage(error)
        })
    } else {
      onMessage('Permission denied')
    }
  }

  async function cancelScan() {
    await cancel()
    scanning = false
    onMessage('cancelled')
  }
</script>

<div class="full-height">
  <div class:invisible={scanning}>
    <div>
      <input type="checkbox" id="scanner-windowed" bind:checked={windowed} />
      <label for="scanner-windowed">Windowed</label>
    </div>
    <div>
      <select class="input" id="format" multiple bind:value={formats}>
        {#each supportedFormats as f}
          <option value={f}>{f}</option>
        {/each}
      </select>
    </div>
    <button class="btn" type="button" on:click={startScan}>Scan</button>
  </div>
  <div class="scanning full-height" class:invisible={!scanning}>
    <div class="scanner-background">
      <!-- this background simulates the camera view -->
    </div>
    <div class="container full-height">
      <div class="barcode-scanner--area--container">
        <div class="relative">
          <p>Aim your camera at a QR code</p>
          <button class="btn" type="button" on:click={cancelScan}>Cancel</button
          >
        </div>
        <div class="square surround-cover">
          <div class="barcode-scanner--area--outer surround-cover">
            <div class="barcode-scanner--area--inner"></div>
          </div>
        </div>
      </div>
    </div>
  </div>
</div>

<style>
  .invisible {
    display: none;
  }

  .full-height {
    height: 100%;
  }

  p {
    color: #fff;
    font-family: sans-serif;
    text-align: center;
    font-weight: 600;
  }

  .container {
    width: 100%;
    height: 100%;
    overflow: hidden;
  }
  .container {
    display: flex;
  }

  .relative {
    position: relative;
    display: flex;
    flex-direction: row;
    justify-content: space-between;
    z-index: 1;
  }

  .square {
    width: 100%;
    position: relative;
    overflow: hidden;
    transition: 0.3s;
  }
  .square:after {
    content: '';
    top: 0;
    display: block;
    padding-bottom: 100%;
  }
  .square > div {
    position: absolute;
    top: 0;
    left: 0;
    bottom: 0;
    right: 0;
  }

  .surround-cover {
    box-shadow: 0 0 0 99999px rgba(0, 0, 0, 0.5);
  }

  .barcode-scanner--area--container {
    width: 80%;
    max-width: min(500px, 80vh);
    margin: auto;
  }
  .barcode-scanner--area--outer {
    display: flex;
    border-radius: 1em;
  }
  .barcode-scanner--area--inner {
    width: 100%;
    margin: 1rem;
    border: 2px solid #fff;
    box-shadow:
      0px 0px 2px 1px rgb(0 0 0 / 0.5),
      inset 0px 0px 2px 1px rgb(0 0 0 / 0.5);
    border-radius: 1rem;
  }

  .scanner-background {
    background: linear-gradient(45deg, #673ab7, transparent);
    background-position: 45% 50%;
    background-size: cover;
    background-repeat: no-repeat;
  }
</style>
