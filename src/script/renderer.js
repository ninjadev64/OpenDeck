const { ipcRenderer } = require("electron");

ipcRenderer.on("categories", (event, categories) => {
    Array.from(document.getElementsByClassName("action-selector")).forEach((selector) => {
        for (const [category, actions] of Object.entries(categories)) {
            let group = document.createElement("optgroup");
            group.label = category;
            actions.forEach((action) => {
                let option = document.createElement("option");
                option.value = action.uuid;
                option.innerText = action.name;
                group.appendChild(option);
            });
            selector.appendChild(group);
        }

        selector.addEventListener("change", () => {
            ipcRenderer.send("keyUpdate", parseInt(selector.id), selector.value);
        });
    });
});

document.getElementById("open-settings").addEventListener("click", () => {
    window.open("settings.html", undefined, "nodeIntegration=yes,contextIsolation=no");
});