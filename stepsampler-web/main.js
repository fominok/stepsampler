let files = [];

function reset() {
  files = [];
  document.getElementById("num-files-selected").textContent = 'No files selected';
  document.getElementById("start-button").disabled = true;
}

function showInfo() {
  document.getElementById("info-alert").style.display = "block";
  document.getElementById("danger-alert").style.display = "none";
}

function showDanger() {
  document.getElementById("danger-alert").style.display = "block";
  document.getElementById("info-alert").style.display = "none";
}

function processFiles() {
  let result;
  try {
    const rateInput = document.getElementById("out-rate").value.trim();
    const silenceThresholdInput = document.getElementById("silence-threshold").value.trim();
    const bitDepthInput = document.getElementById("bit-depth").value.trim();
    const rate = rateInput ? (isNaN(parseInt(rateInput)) ? null : parseInt(rateInput)) : null;
    const silenceThreshold = silenceThresholdInput ? (isNaN(parseFloat(silenceThresholdInput)) ? null : parseFloat(silenceThresholdInput)) : null;
    const bitDepth = bitDepthInput ? (isNaN(parseInt(bitDepthInput)) ? null : parseInt(bitDepthInput)) : null;
    const stereo = document.getElementById("stereo").checked;

    result = window.wasmBindings.process_files(files, rate, silenceThreshold, stereo, bitDepth);
  } catch (error) {
    console.error(error);
    showDanger();
    reset();
    return;
  }

  const blob = new Blob([result], { type: "application/octet-stream" });
  const url = URL.createObjectURL(blob);

  const a = document.createElement("a");
  a.href = url;

  a.download = `${files.length}_processed_sample.wav`;
  document.body.appendChild(a);
  a.click();
  a.remove();
  URL.revokeObjectURL(url);
  showInfo();
  reset();
}

addEventListener("TrunkApplicationStarted", (event) => {
  console.log("application started - bindings:", window.wasmBindings, "WASM:", event.detail.wasm);

  const dropzone = document.body;

  dropzone.addEventListener("dragover", e => {
    e.preventDefault();
  });

  dropzone.addEventListener("dragleave", e => {
    e.preventDefault();
  });

  dropzone.addEventListener("drop", async e => {
    e.preventDefault();

    const droppedFiles = Array.from(e.dataTransfer.files);

    for (const file of droppedFiles) {
      const arrayBuffer = await file.arrayBuffer();
      files.push(new Uint8Array(arrayBuffer));
    }

    document.getElementById("start-button").disabled = false;
    document.getElementById("num-files-selected").textContent = `${droppedFiles.length} files selected`;
  });
});
