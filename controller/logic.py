from time import sleep
from MQTT_Listener import MQTT_Listener
import json
from flask import Flask, request
import paho.mqtt.client as mqtt
import simpleaudio as sa
import environment
import threading
import time
from heartbeat_class import Heartbeat

class Logic:
	
	def __init__(self):
		# Connect
		self.client = mqtt.Client()
		self.client.connect(environment.MQTT_BROKER_HOST, environment.MQTT_BROKER_PORT)
		
		# Setup HTTP via Flask
		self.app = Flask(__name__)
		self.setup_routes()
		
		self.stop_event_audio = threading.Event()
		
		self.flag_first_run = True
		self.health_check_interval = environment.HEALTH_CHECK_INTERVAL_IDEAL
	
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
	def Change_LED_OFF(self, room):
		print("LED " + str(room["room"]) + " OFF")
		my_json = json.dumps({"state": "OFF"})

		topic = room["LED_TOPIC"]		
		print(topic)
		self.client.publish(topic, my_json)
	
	# Change the LED to ON
	def Change_LED_ON(self, room, my_json):
		print("LED " + str(room["room"]) + " ON")
		
		topic = room["LED_TOPIC"]		
		
		self.client.publish(topic, json.dumps(my_json))
			
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
		
		while not self.stop_event_audio.is_set():  # Changed to while not stop_event.is_set()
			play_obj = wave_obj.play()
			play_obj.wait_done()
			
	# Add a function to cleanly stop the audio
	def stopAudio(self):  
		self.stop_event_audio.set()

	def heartbeat_scheduler(self):
		my_heartbeat = Heartbeat()
		while True:
			heartbeat_thread = threading.Thread(target = my_heartbeat.heartbeat)
			heartbeat_thread.start()
			heartbeat_thread.join()
			
			# Sleep to wait for the server answer back what the state is
			sleep(10)
			time.sleep(self.health_check_interval) 

	# Handle the state we are in
	def handleState(self, data):
		state = data["new_state"]
		room = data["current_room_pir"]
		
		print("CURRENT STATE: " + str(state))
		print("Resident is in room " + str(room))
		
		if (state == "Initialization"):
			threading.Thread(target=self.heartbeat_scheduler).start()
			self.flag_first_run = True

		# If we are in "standby", "attended" or "unattended" do the same thing
		elif (state == "Standby" or state == "Attended" or state == "Unattended"):
			self.health_check_interval = environment.HEALTH_CHECK_INTERVAL_IDEAL
			for i in range(len(environment.ROOMS)):
				self.Change_LED_OFF(environment.ROOMS[i])

			self.Power_plug_ON()
			self.stopAudio()
			
			# Start the controller
			print("Starting the controller")
			if (self.flag_first_run):
				MQTT_listener = MQTT_Listener()
				MQTT_listener.start()
				self.flag_first_run = False

		
		# Going into alarmed state
		elif (state == "Alarmed"):
			for i in range(len(environment.ROOMS)):
				self.Change_LED_ON(environment.ROOMS[i], environment.ALARMED_COLOR)
			
			self.Power_plug_ON()
			
			self.stop_event_audio.clear()  # Ensure the stop event is cleared before starting the audio
			threading.Thread(target=self.playAudio).start()
		
		# Going into critically alarmed state
		elif (state == "CriticallyAlarmed"):
			for i in range(len(environment.ROOMS)):
				self.Change_LED_ON(environment.ROOMS[i], environment.CRITICALLY_ALARMED_COLOR)
			
			self.Power_plug_OFF()	
		
		# Going into the faulty state
		elif (state == "Faulty"):
			# Make the LED's pink to visualize the setup went wrong
			for i in range(len(environment.ROOMS)):
				self.Change_LED_ON(environment.ROOMS[i], environment.FAULTY_COLOR)

			self.health_check_interval = environment.FAULTY_HEALTH_CHECK_INTERVAL

