# Start up time
SLEEP_TIME = 1

HTTP_HOST = "http://192.168.43.112:8080"
MQTT_BROKER_HOST = "localhost"
MQTT_BROKER_PORT = 1883

DB_ENDPOINT = HTTP_HOST + "/api/save"
STATUS_ENDPOINT = HTTP_HOST + "/api/status"
HEALTH_CHECK_ENDPOINT = HTTP_HOST + "/api/health_check"
STARTUP_ENDPOINT = HTTP_HOST + "/api/initialization"

STATE_LISTENER_ENDPOINT = "/state_listener"
LISTENER_PORT = 9000
AUDIO_FILE_PATH = "kitchenGuardAudio8000Hz.wav"

ZIGBEE_METADATA_TOPIC = "zigbee2mqtt/bridge/definitions"
INTERVIEW_RESPONSE_TOPIC = "zigbee2mqtt/bridge/response/device/interview"
INTERVIEW_REQUEST_TOPIC = "zigbee2mqtt/bridge/request/device/interview"

BRIDGE_HEALTH_RESPONSE_TOPIC = "zigbee2mqtt/bridge/response/health_check"
BRIDGE_HEALTH_REQUEST_TOPIC = "zigbee2mqtt/bridge/request/health_check"

SENSOR_DICT = {
	"0x54ef44100094740b": "kitchen_pir",
	"0x842e14fffe9e2d85": "LED",
	"0x680ae2fffe7249ff": "PowerPlug"
}

DEVICES = list(SENSOR_DICT.keys())

DEVICES_NON_KITCHEN = [device for device in SENSOR_DICT.values() if device != "kitchen_pir" and device != "LED" and device != "PowerPlug"]

ZIGBEE_LED_TOPIC = "zigbee2mqtt/LED/set"
ZIGBEE_POWERPLUG_TOPIC = "zigbee2mqtt/PowerPlug/set"

# Time between heartbeat (seconds)
HEALTH_CHECK_INTERVAL = 1800

GATEWAY_ID = 13
RES_ID = "RES1"
