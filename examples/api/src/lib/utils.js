export function arrayBufferToBase64(buffer, callback) {
  const blob = new Blob([buffer], {
    type: "application/octet-binary",
  });
  const reader = new FileReader();
  reader.onload = function (evt) {
    const dataurl = evt.target.result;
    callback(dataurl.substr(dataurl.indexOf(",") + 1));
  };
  reader.readAsDataURL(blob);
}
