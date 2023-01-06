const { app, BrowserWindow, Tray, Menu } = require("electron");
const path = require("path");

let isQuitting = false;
let tray;

const createWindow = () => {
	const win = new BrowserWindow({
		webPreferences: {
			nodeIntegration: true,
			contextIsolation: false,
		},
		autoHideMenuBar: true,
		icon: "icon.png"
	});
  
	win.loadFile("index.html");

	let userDataPath = app.getPath("userData");
	win.webContents.executeJavaScript(`localStorage.setItem("userData", \`${userDataPath}\`);`);

	tray = new Tray(path.join(__dirname, "icon.png"));

	tray.setContextMenu(Menu.buildFromTemplate([
		{
			label: "Open", click: () => {
				win.show();
			}
		},
		{
			label: "Quit", click: () => {
				isQuitting = true;
				app.quit();
			}
		}
	]));

	win.on("close", (event) => {
		if (!isQuitting) {
			event.preventDefault();
			win.hide();
			event.returnValue = false;
		}
	});
}

app.whenReady().then(() => {
	createWindow();
  
	app.on("activate", () => {
		if (BrowserWindow.getAllWindows().length === 0) createWindow()
	});
});

app.on("before-quit", () => {
	isQuitting = true;
});