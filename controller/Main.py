from time import sleep
from threading import Thread
from heartbeat_class import Heartbeat
from logic import Logic
import time
import environment
import requests
import paho.mqtt.client as mqtt
from types import SimpleNamespace

# Shared variable between Logic and Main to control time between heartbeats
shared_data = SimpleNamespace(health_check_interval=environment.HEALTH_CHECK_INTERVAL_IDEAL)

def scheduler(my_heartbeat, shared_data):
    while True:
        heartbeat_thread = Thread(target = my_heartbeat.heartbeat)
        heartbeat_thread.start()
        heartbeat_thread.join()
        
        # Sleep to wait for the system to enter standby or faulty state 
        sleep(10)
        time.sleep(shared_data.health_check_interval) 
    
if __name__ == "__main__":
    my_heartbeat = Heartbeat()
    sleep(environment.SLEEP_TIME) # Time for npm to start
    
    # Connect to the client
    client = mqtt.Client()
    client.connect(environment.MQTT_BROKER_HOST, environment.MQTT_BROKER_PORT)
    # Start the logic
    my_logic = Logic(shared_data)
    Thread(target=my_logic.start).start()
    
    # Setup the payload to the server that we are ready to go into "Initialization" mode along with other information
    payload = {
        "res_id": environment.RES_ID,
        "kitchen_pir": environment.KITCHEN_PIR,
        "power_plug": environment.POWER_PLUG,
        "other_pir": environment.OTHER_PIR_DEVICES,
        "led": [environment.LIVING_ROOM_LED, environment.BATHROOM_LED]
    }   
    # Send the payload
    requests.post(environment.STARTUP_ENDPOINT, json=payload)

    # Start the heartbeat
    Thread(target=scheduler, args=(my_heartbeat, shared_data)).start()
    
    # Sleep until the interview is complete
    while (not my_heartbeat.startup_Check):
        sleep(1)
    
    # Keep waiting
    while True:
        sleep(1)
