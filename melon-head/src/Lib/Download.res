let triggerDownload: (
  string,
  string,
  string,
) => unit = %raw(`function(filename, content, mimeType) {
  var blob = new Blob([content], { type: mimeType });
  var url = URL.createObjectURL(blob);
  var a = document.createElement('a');
  a.href = url;
  a.setAttribute('download', filename);
  document.body.appendChild(a);
  a.click();
  document.body.removeChild(a);
  // Mobile Safari aborts the download if the URL is revoked synchronously.
  setTimeout(function() { URL.revokeObjectURL(url); }, 1000);
}`)

let csv = (~filename, ~content) => triggerDownload(filename, content, "text/csv;charset=utf-8;")

let vcard = (~filename, ~content) => triggerDownload(filename, content, "text/vcard;charset=utf-8")
