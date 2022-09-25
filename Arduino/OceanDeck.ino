#include "ArduinoJson.h";

int button = 0;

void setup() {
  Serial.begin(57600);
}

void loop() {
  StaticJsonDocument<16> doc;
  doc["button"] = 0;
  int old = button;
  int dat = analogRead(A0);
  if (dat > 950 && dat < 970) {
    button = 1;
  } else if (dat > 970 && dat < 990) {
    button = 2;
  } else if (dat > 990 && dat < 1010) {
    button = 3;
  } else {
    button = 0;
  }
  if (old != button && button != 0) {
    doc["button"] = button;
    String output = "";
    serializeJson(doc, output);
    Serial.println(output);
  }
  delay(10);
}
