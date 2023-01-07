const Store = require("electron-store");
const store = new Store();

let serialSelect = document.getElementById("serial-port");
store.get("allPorts").forEach((port) => {
    if (port.vendorId === "2341" && port.productId === "0043") {
        serialSelect.insertAdjacentHTML("beforeend", `<option value=${port.path}> ${port.path} </option>`);
    }
});
const options = {
    "serialPort": serialSelect
}
for (const [key, value] of Object.entries(options)) {
    value.value = localStorage.getItem(key);
}
function applyChanges() {
    for (const [key, value] of Object.entries(options)) {
        store.set(key, value.value);
    }
    alert("Changes have been applied. You may need to restart OceanDesktop for them to take effect.");
}
document.getElementById("apply-changes").addEventListener("click", applyChanges);