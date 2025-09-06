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

    const files = Array.from(e.dataTransfer.files);
    const buffers = [];

    for (const file of files) {
      const arrayBuffer = await file.arrayBuffer();
      buffers.push(new Uint8Array(arrayBuffer));
    }

    // Pass the array of Uint8Arrays to WASM once
    const result = window.wasmBindings.process_files(buffers); // WASM expects JsValue/array of Uint8Array

    // Convert returned bytes into a downloadable file
    const blob = new Blob([result], { type: "application/octet-stream" });
    const url = URL.createObjectURL(blob);

    const a = document.createElement("a");
    a.href = url;
    a.download = "processed_sample.wav";
    document.body.appendChild(a);
    a.click();
    a.remove();
    URL.revokeObjectURL(url);
  });
});
