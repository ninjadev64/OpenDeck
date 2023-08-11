#include "ArduinoJson.h";

int lastKey;
int lastSlider0;
int lastSlider1;

int keys[] = { 15, 27, 4, 5, 18, 19, 21, 22, 23 };

void setup() {
  for (int i = 0; i < 9; i++) {
    pinMode(keys[i], INPUT_PULLUP);
    digitalWrite(keys[i], HIGH);
  }

  pinMode(34, INPUT);
  pinMode(35, INPUT);

  Serial.begin(57600);
}

void loop() {
  StaticJsonDocument<16> doc;

  int key = 0;
  for (int i = 0; i < 9; i++) {
    if (digitalRead(keys[i]) == LOW) {
      key = i + 1;
      break;
    }
  }
  if (key != lastKey) {
    lastKey = key;
    doc["key"] = key;
  }

  int s0 = analogRead(34) / 21.328125;
  if (abs(s0 - lastSlider0) > 5) {
    lastSlider0 = s0;
    doc["slider0"] = round(s0);
  }

  int s1 = analogRead(35) / 21.328125;
  if (abs(s1 - lastSlider1) > 5) {
    lastSlider1 = s1;
    doc["slider1"] = round(s1);
  }

  String output = "";
  serializeJson(doc, output);
  if (output == "null") return;
  Serial.println(output);

  delay(50);
}
