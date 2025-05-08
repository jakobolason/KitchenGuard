from time import sleep
from Cep2Controller import Cep2Controller
from Cep2Model import Cep2Model, Cep2ZigbeeDevice
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

class Logic:
	
	def __init__(self):
		
		self.client = mqtt.Client()
		self.client.connect(environment.MQTT_BROKER_HOST, environment.MQTT_BROKER_PORT)
		
		self.app = Flask(__name__)
		self.setup_routes()
		
	def setup_routes(self):
		@self.app.route(environment.STATE_LISTENER_ENDPOINT, methods=['POST'])
		def requests():
			data = request.get_json()
			
			if not data:
				return 'Invalid request', 400
			
			self.handleState(data)
			
			return f"Message Received: {data}", 200
			
		@self.app.route(environment.AUDIO_ENDPOINT, methods=['POST'])
		def handelAudioRequest():

			self.playAudio()
			
			return "Playing Audio", 200
	
	def start(self):
		self.app.run(host='0.0.0.0', port = environment.LISTENER_PORT)
		
	def Change_LED_OFF(self):
		print("LED OFF")
		my_json = json.dumps({"state": "OFF"})
		self.client.publish(environment.ZIGBEE_LED_TOPIC, my_json)
		
	def Change_LED_ON(self):
		print("LED ON")
		my_json = json.dumps({"state": "ON"})
		self.client.publish(environment.ZIGBEE_LED_TOPIC, my_json)
	
	def Power_plug_OFF(client):
		print("PowerPlug OFF")
		my_json = json.dumps({"state": "OFF"})
		self.client.publish(environment.ZIGBEE_POWERPLUG_TOPIC, my_json)
		
	def Power_plug_ON(client):
		print("PowerPlug ON")
		my_json = json.dumps({"state": "ON"})
		self.client.publish(environment.ZIGBEE_POWERPLUG_TOPIC, my_json)
	

	def handleState(self, state):
		if (state == "Standby" or state == "Attended" or state == "Unattended"):
			self.Change_LED_OFF()
			self.Power_plug_ON()
		
		elif (state == "Alarmed"):
			self.Change_LED_ON()
			self.Power_plug_ON()
		
		elif (state == "CriticallyAlarmed"):
			self.Change_LED_ON()
			self.Power_plug_OFF()
	
	def playAudio(self):
		wave_obj = sa.WaveObject.from_wave_file(environment.AUDIO_FILE_PATH)
		play_obj = wave_obj.play()
		play_obj.wait_done()


