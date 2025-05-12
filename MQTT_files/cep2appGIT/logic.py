from time import sleep
from Cep2Controller import Cep2Controller
from threading import Thread
from paho.mqtt.client import Client as MqttClient, MQTTMessage
from paho.mqtt import publish, subscribe
import json
import re
import requests
from flask import Flask, request
import paho.mqtt.client as mqtt
import simpleaudio as sa
import environment
import threading

class Logic:
	
	def __init__(self):
		
		# Connect
		self.client = mqtt.Client()
		self.client.connect(environment.MQTT_BROKER_HOST, environment.MQTT_BROKER_PORT)
		
		# Setup HTTP via Flask
		self.app = Flask(__name__)
		self.setup_routes()
		
		self.stop_event = threading.Event()
	
	# Setup the Raspberry PI endpoint for the state to change
	def setup_routes(self):
		@self.app.route(environment.STATE_LISTENER_ENDPOINT, methods=['POST'])
		def requests():
			data = request.get_json()
			
			if not data:
				return 'Invalid request', 400
			
			self.handleState(data)
			
			return f"Message Received: {data}", 200

	
	# Setup the endpoint
	def start(self):
		self.app.run(host='0.0.0.0', port = environment.LISTENER_PORT)
	
	# Change the LED to off
	def Change_LED_OFF(self):
		print("LED OFF")
		my_json = json.dumps({"state": "OFF"})
		self.client.publish(environment.ZIGBEE_LED_TOPIC, my_json)
	
	# Change the LED to ON
	def Change_LED_ON(self):
		print("LED ON")
		my_json = json.dumps({"state": "ON"})
		self.client.publish(environment.ZIGBEE_LED_TOPIC, my_json)
	
	# Change the powerplug to OFF
	def Power_plug_OFF(self):
		print("PowerPlug OFF")
		my_json = json.dumps({"state": "OFF"})
		self.client.publish(environment.ZIGBEE_POWERPLUG_TOPIC, my_json)
		
	# Change the powerplug to ON
	def Power_plug_ON(self):
		print("PowerPlug ON")
		my_json = json.dumps({"state": "ON"})
		self.client.publish(environment.ZIGBEE_POWERPLUG_TOPIC, my_json)

	# Function to play audio in alarmed state
	def playAudio(self):
		wave_obj = sa.WaveObject.from_wave_file(environment.AUDIO_FILE_PATH)
		
		while not self.stop_event.is_set():
			play_obj = wave_obj.play()
			play_obj.wait_done()

	# Handle the state we are in
	def handleState(self, state):
		print("CURRENT STATE: " + str(state))
		# If we are in "standby", "attended" or "unattended" do the same thing
		if (state == "Standby" or state == "Attended" or state == "Unattended"):
			self.Change_LED_OFF()
			self.Power_plug_ON()
			self.stop_event.set()
		
		# Going into alarmed state
		elif (state == "Alarmed"):
			self.Change_LED_ON()
			self.Power_plug_ON()
			threading.Thread(target=self.playAudio).start()
		
		# Going into critically alarmed state
		elif (state == "CriticallyAlarmed"):
			self.Change_LED_ON()
			self.Power_plug_OFF()	



