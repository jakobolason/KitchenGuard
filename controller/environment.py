# Start up time
SLEEP_TIME = 15

HTTP_HOST = "http://192.168.43.112:8080"
MQTT_BROKER_HOST = "localhost"
MQTT_BROKER_PORT = 1883

DB_ENDPOINT = HTTP_HOST + "/api/save"
STATUS_ENDPOINT = HTTP_HOST + "/api/status"
HEALTH_CHECK_ENDPOINT = HTTP_HOST + "/api/health_check"
STARTUP_ENDPOINT = HTTP_HOST + "/api/initialization"

STATE_LISTENER_ENDPOINT = "/state_listener"
LISTENER_PORT = 9000
AUDIO_FILE_PATH = "/home/raspberry/Desktop/KitchenGuard_newest/KitchenGuard/MQTT_files/cep2appGIT/kitchenGuardAudio8000Hz.wav"

ZIGBEE_METADATA_TOPIC = "zigbee2mqtt/bridge/definitions"
INTERVIEW_RESPONSE_TOPIC = "zigbee2mqtt/bridge/response/device/interview"
INTERVIEW_REQUEST_TOPIC = "zigbee2mqtt/bridge/request/device/interview"

BRIDGE_HEALTH_RESPONSE_TOPIC = "zigbee2mqtt/bridge/response/health_check"
BRIDGE_HEALTH_REQUEST_TOPIC = "zigbee2mqtt/bridge/request/health_check"

SENSOR_DICT = {
	"0x54ef44100094740b": "kitchen_pir",
	"0x54ef441000948cbd": "living_room_pir",
	"0x00158d0005729f18": "bathroom_pir",
	"0x842e14fffe9e2d85": "bathroom_LED",
	"0x60a423fffe02319c": "living_room_LED",
	"0x54ef4410008b372e": "power_plug"
}

DEVICES = list(SENSOR_DICT.keys())

OTHER_PIR_DEVICES = [device for device in SENSOR_DICT.values() if device != "kitchen_pir" and device != "power_plug" and device != "bathroom_LED" and device != "living_room_LED"]

ZIGBEE_LIVING_ROOM_LED_TOPIC = "zigbee2mqtt/living_room_LED/set"
ZIGBEE_BATHROOM_LED_TOPIC = "zigbee2mqtt/bathroom_LED/set"
ZIGBEE_POWERPLUG_TOPIC = "zigbee2mqtt/power_plug/set"

# Time between heartbeat (seconds)
HEALTH_CHECK_INTERVAL = 1800
FAULTY_HEALTH_CHECK_INTERVAL = 30

# Time to wait for errors
HEALTH_CHECK_WAIT = 30

GATEWAY_ID = 13
RES_ID = "RES1"
