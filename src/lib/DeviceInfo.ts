export type DeviceInfo = {
	id: string;
	name: string;
	rows: number;
	columns: number;
	encoders: number;
	encoder_position: "top" | "bottom";
	touchpoints: number;
	infobars: number;
	type: number;
};
