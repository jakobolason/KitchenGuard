# CHANGE THESE
HTTP_HOST = "http://172.20.10.9:8080"

# Sensor names
KITCHEN_PIR = "kitchen_pir"
ROOM_1_PIR = "bathroom_PIR"
ROOM_2_PIR = "living_room_pir"
ROOM_3_PIR = "office_pir"
ROOM_4_PIR = "basement_pir"

#LEDs
ROOM_1_LED = "bathroom_LED"
ROOM_2_LED = "living_room_LED"
ROOM_3_LED = "office_LED"
ROOM_4_LED = "basement_LED"

# Power plug
POWER_PLUG = "power_plug"

SENSOR_DICT = {
	"0x54ef44100094740b": KITCHEN_PIR,
	"0x54ef441000948cbd": ROOM_1_PIR,
	"0x00158d0005729f18": ROOM_2_PIR,
	"0x842e14fffe9e2d85": ROOM_1_LED,
	"0x60a423fffe02319c": ROOM_2_LED,
	"0x54ef4410008b372e": POWER_PLUG,
	"0x89403e1248131232": ROOM_3_LED,
	"0x85e2638491838178": ROOM_3_PIR,
	"0x34895487384e3494": ROOM_4_LED,
	"0x48e2d8484839c389": ROOM_4_PIR
}





# DO NOT TOUCH

LEDs = [ROOM_1_LED, ROOM_2_LED, ROOM_3_LED, ROOM_4_LED]
PIRs = [ROOM_1_PIR, ROOM_2_PIR, ROOM_3_PIR, ROOM_4_PIR]

# Create the rooms
ROOMS = []
for i in range(len(LEDs)):
    if (LEDs[i] and PIRs[i]):
        ROOMS.append({
            "room": "ROOM_" + str(i + 1),
            "LED": LEDs[i],
            "PIR": PIRs[i],
            "LED_TOPIC": "zigbee2mqtt/" + LEDs[i] + "/set",
        })


DEVICES = list(SENSOR_DICT.keys())

OTHER_PIR_NAMES = [s for s in PIRs if s]
OTHER_PIR_DEVICES = [k for k, v in SENSOR_DICT.items() if v in OTHER_PIR_NAMES]

ZIGBEE_POWERPLUG_TOPIC = "zigbee2mqtt/" + POWER_PLUG + "/set"

# Time between heartbeat (seconds)
HEALTH_CHECK_INTERVAL_IDEAL = 1800
FAULTY_HEALTH_CHECK_INTERVAL = 30

# Time to wait for errors
HEALTH_CHECK_WAIT = 30

GATEWAY_ID = 13
RES_ID = "RES1"

# Color codes for LED's
FAULTY_COLOR = {"state": "ON", "color": {"r": 255, "g": 0, "b": 255}}
ALARMED_COLOR = {"state": "ON", "color": {"r": 255, "g": 165, "b": 0}}
CRITICALLY_ALARMED_COLOR = {"state": "ON", "color": {"r": 255, "g": 0, "b": 0}}


MQTT_BROKER_HOST = "localhost"
MQTT_BROKER_PORT = 1883

DB_ENDPOINT = HTTP_HOST + "/api/save"
STATUS_ENDPOINT = HTTP_HOST + "/api/status"
HEALTH_CHECK_ENDPOINT = HTTP_HOST + "/api/health_check"
STARTUP_ENDPOINT = HTTP_HOST + "/api/initialization"

STATE_LISTENER_ENDPOINT = "/state_listener"
LISTENER_PORT = 9000
AUDIO_FILE_PATH = "/home/raspberry/Desktop/KitchenGuard_newest/KitchenGuard/controller/Alarm_Sound_8000.wav"

ZIGBEE_METADATA_TOPIC = "zigbee2mqtt/bridge/definitions"
INTERVIEW_RESPONSE_TOPIC = "zigbee2mqtt/bridge/response/device/interview"
INTERVIEW_REQUEST_TOPIC = "zigbee2mqtt/bridge/request/device/interview"

BRIDGE_HEALTH_RESPONSE_TOPIC = "zigbee2mqtt/bridge/response/health_check"
BRIDGE_HEALTH_REQUEST_TOPIC = "zigbee2mqtt/bridge/request/health_check"

# Start up time
SLEEP_TIME = 1 # 15
