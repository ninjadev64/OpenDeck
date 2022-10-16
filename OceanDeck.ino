#include "ArduinoJson.h";

void setup() {
  Serial.begin(57600);
}

void loop() {
  StaticJsonDocument<16> doc;
  doc["button"] = analogRead(A0);
  String output = "";
  serializeJson(doc, output);
  Serial.println(output);
  delay(100);
}
