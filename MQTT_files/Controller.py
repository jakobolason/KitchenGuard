import paho.mqtt.client as mqtt
import json
import threading
import time


def on_message(client, userdata, msg):
	my_json = {"state": "OFF"}
	new_my_json = json.dumps(my_json)
	client.publish("zigbee2mqtt/LED/set", new_my_json)

def PIR_light():
	client = mqtt.Client()
	client.on_message = on_message
	client.connect("localhost", 1883)
	client.subscribe("zigbee2mqtt/PIR")
	time.sleep(3)
	print("slept for 3 seconds")
	#client.loop_forever()

#t1 = threading.Thread(target=PIR_light)
#t1.start()

client = mqtt.Client()
client.connect("localhost", 1883)
client.subscribe("zigbee2mqtt/PIR")
client.on_message = PIR_light
my_json = {"state": "OFF"}
#new_my_json = json.dumps(my_json)
#client.publish("zigbee2mqtt/PowerPlug/set", new_my_json)
#t1.join()



client.loop_forever()

client.disconnect()
