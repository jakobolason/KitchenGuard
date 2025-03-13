import paho.mqtt.client as mqtt
import json
import threading
import time


def on_message(client, userdata, msg):
	my_json = {"state": "TOGGLE"}
	new_my_json = json.dumps(my_json)
	print("Creating message")
	client.publish("zigbee2mqtt/LED/set", new_my_json)

def PIR_light(client):
	print("Pir light")
	client.on_message = PIR_light
	client.on_message = on_message
	#client.connect("localhost", 8080)
	#client.subscribe("zigbee2mqtt/PIR")
	#time.sleep(3)
	print("slept for 3 seconds")
	#client.loop_forever()

def LED_light(client):
	print("Toggling led")
	color_dump = json.dumps({"state": "TOGGLE"})
	client.publish("zigbee2mqtt/LED/set", color_dump)
	for i in range(4):
		color_dump = json.dumps({"state": "TOGGLE"})
		client.publish("zigbee2mqtt/LED/set", color_dump)
		time.sleep(1)

def Power_plug(client):
	my_json = {"state": "OFF"}
	new_my_json = json.dumps(my_json)
	client.publish("zigbee2mqtt/PowerPlug/set", new_my_json)
	print("published to power plug")

if __name__ == "__main__":
	print("here")
	#t1 = threading.Thread(target=PIR_light)
	#t1.start()
	#print("t1 started")
	
	client = mqtt.Client()
	client.connect("localhost", 1883)
	client.subscribe("zigbee2mqtt/LED")
	
	LED_light(client)
	Power_plug(client)
	print("Getting")
#	client.publish("zigbee2mqtt/LED/get")
	#t1.join()
	#client.loop_forever()
	client.disconnect()
