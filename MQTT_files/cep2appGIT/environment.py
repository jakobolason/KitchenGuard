
SLEEP_TIME = 1

HTTP_HOST = "http://192.168.1.170:8080"
MQTT_BROKER_HOST = "localhost"
MQTT_BROKER_PORT = 1883

DB_ENDPOINT = HTTP_HOST + "/api/save"
STATUS_ENDPOINT = HTTP_HOST + "/api/status"
HEALTH_CHECK_ENDPOINT = HTTP_HOST + "/api/health_check"

STATE_LISTENER_ENDPOINT = "/state_listener"
LISTENER_PORT = 9000
AUDIO_ENDPOINT = "/audio"
AUDIO_FILE_PATH = "kitchenGuardAudio8000Hz.wav"

ZIGBEE_METADATA_TOPIC = "zigbee2mqtt/bridge/definitions"
INTERVIEW_RESPONSE_TOPIC = "zigbee2mqtt/bridge/response/device/interview"
INTERVIEW_REQUEST_TOPIC = "zigbee2mqtt/bridge/request/device/interview"

BRIDGE_HEALTH_RESPONSE_TOPIC = "zigbee2mqtt/bridge/response/health_check"
BRIDGE_HEALTH_REQUEST_TOPIC = "zigbee2mqtt/bridge/request/health_check"

SENSOR_DICT = {
	"0x00158d0005729f18": "PIR",
	"0x842e14fffe9e2d85": "LED",
	"0x680ae2fffe7249ff": "PowerPlug"
}


DEVICES = list(SENSOR_DICT.keys())

ZIGBEE_LED_TOPIC = "zigbee2mqtt/LED/set"
ZIGBEE_POWERPLUG_TOPIC = "zigbee2mqtt/PowerPlug/set"

# Time between heartbeat (seconds)
HEALTH_CHECK_INTERVAL = 1800

GATEWAY_ID = 13
RES_ID = "RES1"
