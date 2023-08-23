if (require("electron-squirrel-startup")) return;

const { app, ipcMain, BrowserWindow, Tray, Menu } = require("electron");
const path = require("path");

if (!app.requestSingleInstanceLock()) {
	app.exit();
}

const { allActions, categories, setProfile, updateSlot, ActionInstance } = require("./shared");
const store = require("./store");

const AutoLaunch = require("auto-launch");

let isQuitting = false;
let tray;

let mainWindow;

function createWindow() {
	mainWindow = new BrowserWindow({
		webPreferences: {
			nodeIntegration: true,
			contextIsolation: false,
		},
		autoHideMenuBar: true,
		icon: path.join(__dirname, "../assets/icon.png")
	});
  
	mainWindow.loadFile(path.join(__dirname, "../markup/index.html"));

	ipcMain.on("createInstance", (_event, action, device, type, position, index) => {
		let devices = store.get("devices");
		let instance = new ActionInstance(allActions[action], device, devices[device].selectedProfile, type, position, index);
		updateSlot(instance.context, instance);
		mainWindow.webContents.send("updateState", instance.context, instance);
	});
	
	ipcMain.on("slotUpdate", (_event, context, instance) => {
		updateSlot(context, instance);
		mainWindow.webContents.send("updateState", context, instance);
	});
	
	ipcMain.on("requestCategories", () => {
		mainWindow.webContents.send("categories", categories);
	});

	ipcMain.on("requestDevices", () => {
		mainWindow.webContents.send("devices", store.get("devices"));
	});

	ipcMain.on("requestProfiles", (_event, device) => {
		let d = store.get("devices")[device];
		mainWindow.webContents.send("profiles", d.profiles, d.selectedProfile);
	});

	ipcMain.on("createProfile", (_event, device, name, id) => {
		let devices = store.get("devices");
		devices[device].profiles[id] = {
			name,
			key: [ [ null ], [ null ], [ null ], [ null ], [ null ], [ null ], [ null ], [ null ], [ null ] ],
			slider: [ [ null ], [ null ] ]
		};
		store.set("devices", devices);
		mainWindow.webContents.send("profiles", devices[device].profiles, devices[device].selectedProfile);
	});

	ipcMain.on("profileUpdate", (_event, device, id) => {
		setProfile(device, id);
		mainWindow.webContents.send("profiles", store.get("devices")[device].profiles, id);
	});

	mainWindow.on("close", (event) => {
		if (!isQuitting) {
			event.preventDefault();
			mainWindow.hide();
			event.returnValue = false;
		}
	});

	app.on("activate", () => {
		mainWindow.show();
		mainWindow.restore();
		mainWindow.focus();
	});

	app.on("second-instance", () => {
		mainWindow.show();
		mainWindow.restore();
		mainWindow.focus();
	});
}

app.whenReady().then(() => {
	require("./plugins");
	require("./devices");
	require("./propertyinspector");

	createWindow();

	tray = new Tray(path.join(__dirname, "../assets/icon.png"));
	tray.setContextMenu(Menu.buildFromTemplate([
		{
			label: "Open", click: () => {
				mainWindow.show();
			}
		},
		{
			label: "Quit", click: () => {
				isQuitting = true;
				app.exit();
			}
		}
	]));

	let autoLaunch = new AutoLaunch({
		name: "OpenDeck",
		isHidden: true
	});
	autoLaunch.isEnabled().then((isEnabled) => {
		if (store.get("autoLaunch") && !isEnabled) autoLaunch.enable();
		if (!store.get("autoLaunch") && isEnabled) autoLaunch.disable();
	});
});

app.on("before-quit", () => {
	isQuitting = true;
});

function getMainWindow() {
	return mainWindow;
}

module.exports = { getMainWindow };