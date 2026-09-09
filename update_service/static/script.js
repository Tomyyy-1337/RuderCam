const fileInput = document.getElementById('firmwareFile');
const uploadBtn = document.getElementById('uploadBtn');

// Fetch and display the current firmware version
async function fetchCurrentVersion() {
    try {
        const response = await fetch('/api/current_version');
        if (response.ok) {
            const version = await response.text();
            document.getElementById('currentVersion').textContent = version;
        } else {
            document.getElementById('currentVersion').textContent = 'Error fetching version';
        }
    } catch (error) {
        document.getElementById('currentVersion').textContent = 'Error fetching version';
    }
}

fetchCurrentVersion();

fileInput.addEventListener('change', () => {
    uploadBtn.disabled = !fileInput.files.length;
});

uploadBtn.addEventListener('click', async () => {
    const file = fileInput.files[0];
    if (!file) return;

    uploadBtn.disabled = true;
    const originalText = uploadBtn.textContent;
    uploadBtn.textContent = 'Uploading...';

    try {
        const arrayBuffer = await file.arrayBuffer();
        const response = await fetch('/api/update', {
            method: 'POST',
            body: new Uint8Array(arrayBuffer),
            headers: { 'Content-Type': 'application/octet-stream' }
        });

        if (response.ok) {
            alert('Firmware updated successfully!');
            fileInput.value = '';
            fetchCurrentVersion(); // Refresh the displayed version after a successful update
        } else {
            alert('Upload failed: ' + response.statusText);
        }
    } catch (error) {
        alert('Error: ' + error.message);
    } finally {
        uploadBtn.disabled = false;
        uploadBtn.textContent = originalText;
    }
});