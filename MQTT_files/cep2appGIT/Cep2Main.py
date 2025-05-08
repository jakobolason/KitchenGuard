from time import sleep
from Cep2Controller import Cep2Controller
from Cep2Model import Cep2Model, Cep2ZigbeeDevice
from threading import Thread
from paho.mqtt.client import Client as MqttClient, MQTTMessage
from paho.mqtt import publish, subscribe
import json
from heartbeat_class import Heartbeat
from logic import Logic
import time
import environment

import paho.mqtt.client as mqtt
import threading

def scheduler():
    my_heartbeat = Heartbeat()
    while True:
        threading.Thread(target = my_heartbeat.heartbeat).start()
        time.sleep(environment.HEALTH_CHECK_INTERVAL) 
    
if __name__ == "__main__":
    sleep(environment.SLEEP_TIME) # Time for npm to start
    
    # Create a data model and add a list of known Zigbee devices.
    devices_model = Cep2Model()
    devices_model.add([Cep2ZigbeeDevice("0x00158d0005729f18", "PIR"),
                       Cep2ZigbeeDevice("0x842e14fffe9e2d85", "LED"),
                       Cep2ZigbeeDevice("0x680ae2fffe7249ff", "PowerPlug")])
                       
                       
                       

    # Create a controller and give it the data model that was instantiated.
    controller = Cep2Controller(devices_model)
    
    my_logic = Logic()
    threading.Thread(target=my_logic.start).start()
    
    client = mqtt.Client()
    client.connect(environment.MQTT_BROKER_HOST, environment.MQTT_BROKER_PORT)
    
    controller.start()
    threading.Thread(target = scheduler).start()
    

    while True:
        sleep(1)

    controller.stop()
