const { SerialPort } = require("serialport");
const { ReadlineParser } = require("@serialport/parser-readline");
const port = new SerialPort({ path: localStorage.getItem("serialPort") ? localStorage.getItem("serialPort") : "/dev/ttyACM0", baudRate: 57600 });
const parser = port.pipe(new ReadlineParser({ delimiter: '\r\n' }));

document.getElementById("open-settings").addEventListener("click", () => {
	window.open("settings.html");
});

const basicActions = new BasicActions();
const discordActions = new DiscordActions();

var actions = {
	"clog": { name: "Debug Log", handler: (d) => { basicActions.text(d); } },
	"dmute": { name: "Discord Mute", handler: (d) => { discordActions.mute(d); } },
	"ddeaf": { name: "Discord Deafen", handler: (d) => { discordActions.deafen(d); } },
	"exec": { name: "Launch Executable", handler: (d) => { basicActions.application(d); }, additionalInput: true },
	"keyc": { name: "Key Combination", handler: (d) => { basicActions.keyCombo(d); }, additionalInput: true }
};

var callbacks = {
	"1": (d) => { basicActions.text(d); },
	"2": (d) => { basicActions.text(d); },
	"3": (d) => { basicActions.text(d); }
};

(function() {
	let elements = document.getElementsByClassName("button-dropdown");
	for (let i = 0; i < elements.length; i++) {
		let dropdown = elements[i];
		let optionHTML = "";
		for (const [key, value] of Object.entries(actions)) {
			optionHTML+=`<option value="${key}"> ${value.name} </option>`;
		}
		dropdown.innerHTML = optionHTML;
		dropdown.addEventListener("change", function() {
			let option = document.getElementById(`option${this.id}`);
			option.disabled = true;
			option.value = "";
			callbacks[this.id] = actions[this.value].handler;
			if (actions[this.value].additionalInput) option.disabled = false;
		});
}})();

parser.on("data", function (dat) {
	callbacks[`${JSON.parse(dat).button}`](JSON.parse(dat));
});