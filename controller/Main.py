from time import sleep
from threading import Thread
from logic import Logic
import environment
import requests
import paho.mqtt.client as mqtt
    
if __name__ == "__main__":
    sleep(environment.SLEEP_TIME) # Time for npm to start
    
    # Connect to the client
    client = mqtt.Client()
    client.connect(environment.MQTT_BROKER_HOST, environment.MQTT_BROKER_PORT)
    # Start the logic
    my_logic = Logic()
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