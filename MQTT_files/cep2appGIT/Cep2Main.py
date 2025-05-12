from time import sleep
from Cep2Controller import Cep2Controller
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
        threading.Thread(target = my_heartbeat.heartbeat).start()
        time.sleep(environment.HEALTH_CHECK_INTERVAL) 
    
if __name__ == "__main__":
    my_heartbeat = Heartbeat()
    
    sleep(environment.SLEEP_TIME) # Time for npm to start
                       
    # Create a controller and give it the data model that was instantiated.
    controller = Cep2Controller()
    controller.start()

    
    # Start the logic
    my_logic = Logic()
    threading.Thread(target=my_logic.start).start()
    
    # Connect to the client
    client = mqtt.Client()
    client.connect(environment.MQTT_BROKER_HOST, environment.MQTT_BROKER_PORT)
    
    # Start the heartbeat
    threading.Thread(target = scheduler, args = (my_heartbeat, )).start()
    
    # Sleep until the interview is complete
    while (not my_heartbeat.startup_Check):
        sleep(1)
    
    # Setup the payload to the server that we are ready to go into "standby" mode along with other information
    payload = {
        "res_id": environment.RES_ID,
        "kitchen_pir": "kitchen_pir",
        "power_plug": "PowerPlug",
        "other_pir": environment.DEVICES_NON_KITCHEN,
        "led": ["led"]
    }
    print(payload)
    
    # Send the payload
    print("Sending initialization to server")
    requests.post(environment.STARTUP_ENDPOINT, json=payload)
    print("Initialization sent")
    
    # Keep waiting
    while True:
        sleep(1)

    controller.stop()
