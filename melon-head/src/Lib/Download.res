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

/* Android WebView browsers (DuckDuckGo, ...) ignore the download attribute on
   blob links and appear to derive the extension from the MIME type alone,
   which failed for "text/vcard;charset=utf-8". "text/x-vcard" is the type
   Android maps to ".vcf". Blob strings are encoded as UTF-8 anyway. */
let vcard = (~filename, ~content) => triggerDownload(filename, content, "text/x-vcard")
