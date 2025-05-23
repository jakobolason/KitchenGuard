from time import sleep
from threading import Thread
from paho.mqtt.client import Client as MqttClient
import json
import requests
import environment
import re

class Heartbeat:
	
	# Initialization variables
	def __init__(self):
		self.status = [(v, "error") for v in environment.SENSOR_DICT.values()]
		self.status.append(("Bridge", "ok"))
		self.status.append(("PI", "ok"))

		self.startup_Check = False
	
	# Start the heartbeat operation
	def heartbeat(self):
		sub_thread = Thread(target = self.heartbeat_subscriber)
		sub_thread.start()
		self.heartbeat_interview()

		sub_thread.join()

	# The heatbeat logic
	def heartbeat_subscriber(self):
		# Keeps control of how many interviews we have done
		received_messages = 0
		target_message_count = len(environment.DEVICES)

		# When the interview message comes from each of the sensors
		def on_message(client, userdata, msg):
			nonlocal received_messages
			# Try to get the information from the interviews (this happens when the interview is "ok")
			try:
				payload = msg.payload
				payload_data = json.loads(payload)
				hex_name = payload_data['data']['id']
				ID = environment.SENSOR_DICT[hex_name]
				status = payload_data['status']
			
			# If the interview failed go to this and extract the information
			except (KeyError, json.JSONDecodeError) as e:
				payload = msg.payload.decode("utf-8")
				payload_data = json.loads(payload)
				match = re.search(r'0x[0-9a-fA-F]+', payload) 
				if match:
					hex_name = match.group(0)
				ID = environment.SENSOR_DICT[hex_name]
				status = payload_data['status']		
			
			# Loop though all sensors
			for i in range(len(self.status)):
				if (ID == self.status[i][0]):
					self.status[i] = (self.status[i][0], status)
			
			print(f"topic = {msg.topic}, ID = {ID}, Status = {status}")
			
			# Up the count of the messages recived
			received_messages += 1	
			
			# If we have recived all interviews
			if received_messages >= target_message_count:
				print("✅ Received all messages. Stopping loop...")
				client.loop_stop()	
				client.disconnect()
				print("Interview finished")
		
		# Setup
		client = MqttClient()
		client.on_message = on_message
		client.connect(environment.MQTT_BROKER_HOST, environment.MQTT_BROKER_PORT)
		client.subscribe(environment.INTERVIEW_RESPONSE_TOPIC)
		client.loop_start()

		# Wait until all messages are received (all interviews completed)
		time_waited = 0
		while received_messages < target_message_count and time_waited <= environment.HEALTH_CHECK_WAIT:
			print("Waiting for a sensor to be ok")
			sleep(0.5)
			time_waited += 0.5

		# Setup the payload to the server about the interviews
		event = {
			"res_id": environment.RES_ID,
			"data": self.status
		}
		
		# Send the payload
		response = requests.post(environment.HEALTH_CHECK_ENDPOINT, json=event)
		print("Response sent to webserver: " + str(response))
		self.startup_Check = True

	
	# Setup the interviews
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
			
			
			
