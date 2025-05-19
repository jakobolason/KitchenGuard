# Start up time
SLEEP_TIME = 1 # 15

HTTP_HOST = "http://192.168.43.112:8080"
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

# Sensor names
KITCHEN_PIR = "kitchen_pir"
LIVING_ROOM_PIR = "living_room_pir"
BATHROOM_PIR = "bathroom_pir"
BATHROOM_LED = "bathroom_LED"
LIVING_ROOM_LED = "living_room_LED"
POWER_PLUG = "power_plug"

# Room names
LIVING_ROOM = "living_room"
BATHROOM = "bathroom"

SENSOR_DICT = {
	"0x54ef44100094740b": KITCHEN_PIR,
	"0x54ef441000948cbd": LIVING_ROOM_PIR,
	"0x00158d0005729f18": BATHROOM_PIR,
	"0x842e14fffe9e2d85": BATHROOM_LED,
	"0x60a423fffe02319c": LIVING_ROOM_LED,
	"0x54ef4410008b372e": POWER_PLUG
}

DEVICES = list(SENSOR_DICT.keys())

OTHER_PIR_DEVICES = [device for device in SENSOR_DICT.values() if device != KITCHEN_PIR and device != POWER_PLUG and device != BATHROOM_LED and device != LIVING_ROOM_LED]

ZIGBEE_LIVING_ROOM_LED_TOPIC = "zigbee2mqtt/" + LIVING_ROOM_LED + "/set"
ZIGBEE_BATHROOM_LED_TOPIC = "zigbee2mqtt/" + BATHROOM_LED + "/set"
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
