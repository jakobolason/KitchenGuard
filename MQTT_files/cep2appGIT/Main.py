from time import sleep
from threading import Thread
from paho.mqtt.client import Client as MqttClient, MQTTMessage
from paho.mqtt import publish, subscribe
import json
from heartbeat_class import Heartbeat
from logic import Logic
import time
import environment
import requests


def scheduler(my_heartbeat):
    while True:
        heartbeat_thread = Thread(target = my_heartbeat.heartbeat)
        heartbeat_thread.start()
        heartbeat_thread.join()
        
        # Sleep to wait for the system to enter standby or faulty state 
        sleep(10)
        time.sleep(environment.HEALTH_CHECK_INTERVAL) 
    
if __name__ == "__main__":
    my_heartbeat = Heartbeat()
    sleep(environment.SLEEP_TIME) # Time for npm to start
    
    # Connect to the client
    client = mqtt.Client()
    client.connect(environment.MQTT_BROKER_HOST, environment.MQTT_BROKER_PORT)
    # Start the logic
    my_logic = Logic()
    threading.Thread(target=my_logic.start).start()
    
    # Setup the payload to the server that we are ready to go into "Initialization" mode along with other information
    payload = {
        "res_id": environment.RES_ID,
        "kitchen_pir": "kitchen_pir",
        "power_plug": "power_plug",
        "other_pir": environment.OTHER_PIR_DEVICES,
        "led": ["living_room_LED", "bathroom_LED"]
    }   
    # Send the payload
    requests.post(environment.STARTUP_ENDPOINT, json=payload)

    # Start the heartbeat
    threading.Thread(target = scheduler, args = (my_heartbeat, )).start()   
    
    # Sleep until the interview is complete
    while (not my_heartbeat.startup_Check):
        sleep(1)
    
    # Keep waiting
    while True:
        sleep(1)
