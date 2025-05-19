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
	
	# Get the LED endpoint based on the room
	def getTopicRoomLED(self, room):
		if (room == environment.LIVING_ROOM):
			return environment.ZIGBEE_LIVING_ROOM_LED_TOPIC
		elif (room == environment.BATHROOM):
			return environment.ZIGBEE_BATHROOM_LED_TOPIC
		else: return "error"
	
	# Change the LED to off
	def Change_LED_OFF(self, room):
		print("LED " + str(room) + " OFF")
		my_json = json.dumps({"state": "OFF"})
		
		topic = self.getTopicRoomLED(room)
		
		self.client.publish(topic, my_json)
	
	# Change the LED to ON
	def Change_LED_ON(self, room, my_json):
		print("LED " + str(room) + " ON")
		
		topic = self.getTopicRoomLED(room)
		print(topic)
		
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
			self.Change_LED_OFF(environment.LIVING_ROOM)
			self.Change_LED_OFF(environment.BATHROOM)
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
			if (room == environment.BATHROOM_PIR):
				self.Change_LED_ON(environment.BATHROOM, environment.ALARMED_COLOR)
			elif (room == environment.LIVING_ROOM_PIR):
				self.Change_LED_ON(environment.LIVING_ROOM, environment.ALARMED_COLOR)
			
			self.Power_plug_ON()
			
			self.stop_event_audio.clear()  # Ensure the stop event is cleared before starting the audio
			threading.Thread(target=self.playAudio).start()
		
		# Going into critically alarmed state
		elif (state == "CriticallyAlarmed"):
			self.Change_LED_ON(environment.LIVING_ROOM, environment.CRITICALLY_ALARMED_COLOR)
			self.Change_LED_ON(environment.BATHROOM, environment.CRITICALLY_ALARMED_COLOR)
			self.Power_plug_OFF()	
		
		# Going into the faulty state
		elif (state == "Faulty"):
			# Make the LED's pink to visualize the setup went wrong
			self.Change_LED_ON(environment.LIVING_ROOM, environment.FAULTY_COLOR)
			self.Change_LED_ON(environment.BATHROOM, environment.FAULTY_COLOR)

			self.health_check_interval = environment.FAULTY_HEALTH_CHECK_INTERVAL

