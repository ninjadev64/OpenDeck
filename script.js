const { SerialPort } = require("serialport");
const { ReadlineParser } = require("@serialport/parser-readline");
const spawn = require("child_process").spawn;
const RPC = require("discord-rpc");
const KS = require("node-key-sender");
const port = new SerialPort({ path: "/dev/ttyACM0", baudRate: 57600 });
const parser = port.pipe(new ReadlineParser({ delimiter: '\r\n' }));

var actions = {
	"clog": { name: "Debug Log", handler: text },
	"dmute": { name: "Discord Mute", handler: mute },
	"ddeaf": { name: "Discord Deafen", handler: deafen },
	"exec": { name: "Launch Executable", handler: application, additionalInput: true },
	"keyc": { name: "Key Combination", handler: keyCombo, additionalInput: true }
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

var callbacks = {
	"1": text,
	"2": text,
	"3": text
}

const clientId = "1023592341484863638";
const scopes = [ "rpc", "rpc.voice.read", "rpc.voice.write" ];
const client = new RPC.Client({ transport: 'ipc' });

(async () => {
	let token = localStorage.getItem("discordToken");
	if (token) {
		client.login({ clientId, clientSecret, accessToken: localStorage.getItem("discordToken"), scopes, redirectUri: 'http://localhost:53134' });
	} else {
    	client.login({ clientId, clientSecret, scopes, redirectUri: 'http://localhost:53134' });
	}

    client.on('ready', () => {
        console.log('Authed for Discord user', client?.user?.username);
		localStorage.setItem("discordToken", client?.accessToken);
    });
})();

function text(dat) {
	console.log(`Button ${JSON.parse(dat).button} was pressed!`);
}

function mute(_) {
	client.getVoiceSettings().then((settings) => {
		client.setVoiceSettings({
			mute: !settings.mute
		});
	});
}
function deafen(_) {
	client.getVoiceSettings().then((settings) => {
		client.setVoiceSettings({
			deaf: !settings.deaf
		});
	});
}

function application(dat) {
	button = JSON.parse(dat).button;
	let child = spawn(document.getElementById(`option${button}`).value, [], {
		detached: true,
		stdio: [ 'ignore', 'ignore', 'ignore' ]
	});
	
	child.unref();
}

function keyCombo(dat) {
	button = JSON.parse(dat).button;
	KS.sendCombination(document.getElementById(`option${button}`).value.split("+"));
}

parser.on("data", function (dat) {
	callbacks[`${JSON.parse(dat).button}`](dat);
});