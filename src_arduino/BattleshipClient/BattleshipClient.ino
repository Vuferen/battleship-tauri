#include <Battleship.h>

#define cols 10 // used when determening led index

#define MOTOR_PIN 7
#define BUTTON_PIN 12
#define VRX_PIN  A4
#define VRY_PIN  A5

#define NUM_LEDS 100
#define LED_PIN 3
#define BRIGHTNESS 100

CRGB leds[NUM_LEDS];
FastLED_NeoPixel_Variant strip(leds, NUM_LEDS);

bool isServer = false;
bool invertJoystickAxis = true;

Battleship battleship(isServer, cols, MOTOR_PIN, BUTTON_PIN, VRX_PIN, VRY_PIN, invertJoystickAxis);

void setup()
{
  Serial1.begin(9600);
  Serial.begin(9600);
  strip.begin(FastLED.addLeds<WS2812B, LED_PIN, GRB>(leds, NUM_LEDS));
  strip.setBrightness(BRIGHTNESS);
  strip.clear();
  strip.show();
}

void loop()
{
  battleship.run(strip);
}
