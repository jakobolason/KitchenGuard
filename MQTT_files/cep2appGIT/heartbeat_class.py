from time import sleep
from Cep2Controller import Cep2Controller
from Cep2Model import Cep2Model, Cep2ZigbeeDevice
from threading import Thread
from paho.mqtt.client import Client as MqttClient, MQTTMessage
from paho.mqtt import publish, subscribe
import json
import re
import requests

from Cep2Model import Cep2Model
import environment

class Heartbeat:
	
	def __init__(self):
		self.PIR_status = "error"
		self.LED_status = "error"
		self.PowerPlug_status = "error"
		self.Bridge_status = "ok"
		self.PI_status = "ok"
		
	def heartbeat(self):
		sub_thread = Thread(target = self.heartbeat_subscriber)
		interview_thread = Thread(target = self.heartbeat_interview)
		sub_thread.start()
		interview_thread.start()
		
		
		interview_thread.join()
		sub_thread.join()


	def heartbeat_subscriber(self):

		
		received_messages = 0
		target_message_count = len(environment.DEVICES)

		def on_message(client, userdata, msg):
			nonlocal received_messages
			try:
				payload = msg.payload
				payload_data = json.loads(payload)
				hex_name = payload_data['data']['id']
				ID = environment.SENSOR_DICT[hex_name]
				status = payload_data['status']
				
			except (KeyError, json.JSONDecodeError) as e:
				payload = msg.payload.decode("utf-8")
				payload_data = json.loads(payload)
				match = re.search(r'0x[0-9a-fA-F]+', payload) 
				if match:
					hex_name = match.group(0)
				ID = environment.SENSOR_DICT[hex_name]
				status = payload_data['status']		

			if (ID == "PIR"):
				self.PIR_status = status
			elif (ID == "LED"):
				self.LED_status = status
			elif (ID == "PowerPlug"):
				self.PowerPlug_status = status
			
			print(f"topic = {msg.topic}, ID = {ID}, Status = {status}")
			
			received_messages += 1	
			if received_messages >= target_message_count:
				print("✅ Received all messages. Stopping loop...")
				client.loop_stop()	
				client.disconnect()
				print("Interview finished")

		client = MqttClient()
		client.on_message = on_message
		client.connect(environment.MQTT_BROKER_HOST, environment.MQTT_BROKER_PORT)
		client.subscribe(environment.INTERVIEW_RESPONSE_TOPIC)
		client.loop_start()

		# ✅ Actively wait until all messages are received
		time_waited = 0
		while received_messages < target_message_count and time_waited <= 120:
			print("Waiting for a sensor to be ok")
			sleep(0.5)
			time_waited += 0.1

		
		event = {
			"PIR": self.PIR_status,
			"LED": self.LED_status,
			"PowerPlug": self.PowerPlug_status,
			"Bridge": self.Bridge_status,
			"PI": self.PI_status
		}
		
		response = requests.post(environment.HEALTH_CHECK_ENDPOINT, json=event)
		print("Response sent to webserver")


	def heartbeat_interview(self):

		def on_connect(client, userdata, flags, rc):
			print("Connected to MQTT broker!" if rc == 0 else f"failed to connet, return code {rc}")
		
		client = MqttClient()
		client.on_connect = on_connect

		client.connect(environment.MQTT_BROKER_HOST, environment.MQTT_BROKER_PORT)
		client.loop_start()

		for device in environment.DEVICES:
			payload = {"id": device}
			client.publish(environment.INTERVIEW_REQUEST_TOPIC, json.dumps(payload))
			print(f"Interview request sent for {device}")
			
			
			
