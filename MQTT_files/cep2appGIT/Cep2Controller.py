from Cep2Model import Cep2Model
from Cep2WebClient import Cep2WebClient, Cep2WebDeviceEvent
from Cep2Zigbee2mqttClient import (Cep2Zigbee2mqttClient,
                                   Cep2Zigbee2mqttMessage, Cep2Zigbee2mqttMessageType)
import requests
import json
import datetime

class Cep2Controller:
    HTTP_HOST = "http://192.168.198.236:8080"
    MQTT_BROKER_HOST = "localhost"
    MQTT_BROKER_PORT = 1883

    """ The controller is responsible for managing events received from zigbee2mqtt and handle them.
    By handle them it can be process, store and communicate with other parts of the system. In this
    case, the class listens for zigbee2mqtt events, processes them (turn on another Zigbee device)
    and send an event to a remote HTTP server.
    """

    def __init__(self, devices_model: Cep2Model) -> None:
        """ Class initializer. The actuator and monitor devices are loaded (filtered) only when the
        class is instantiated. If the database changes, this is not reflected.

        Args:
            devices_model (Cep2Model): the model that represents the data of this application
        """
        self.__devices_model = devices_model
        self.__z2m_client = Cep2Zigbee2mqttClient(host=self.MQTT_BROKER_HOST,
                                                  port=self.MQTT_BROKER_PORT,
                                                  on_message_clbk=self.__zigbee2mqtt_event_received)

    def start(self) -> None:
        """ Start listening for zigbee2mqtt events.
        """
        self.__z2m_client.connect()
        print(f"Zigbee2Mqtt is {self.__z2m_client.check_health()}")
        # A new event is sent as an HTTP GET request
        response = requests.get(self.HTTP_HOST + "/api/status")
        if response.status_code == 200:
            print("Status of database was a SUCCESS!")
        else:
            print("Response from request: ", response)
            
        #event = {"first_name": "Jakob", "last_name": "Miller", "username": "Jalle", "email": "jalle@cool.com"}
        
        

    def stop(self) -> None:
        """ Stop listening for zigbee2mqtt events.
        """
        self.__z2m_client.disconnect()

    def __zigbee2mqtt_event_received(self, message: Cep2Zigbee2mqttMessage) -> None:
        """ Process an event received from zigbee2mqtt. This function given as callback to
        Cep2Zigbee2mqttClient, which is then called when a message from zigbee2mqtt is received.

        Args:
            message (Cep2Zigbee2mqttMessage): an object with the message received from zigbee2mqtt
        """
 #       print("incoming message: ", message)
        # If message is None (it wasn't parsed), then don't do anything.
        if not message:
            print("NO MESSAGE RECIEVED")
            return

        print(
            f"zigbee2mqtt event received on topic {message.topic}: {message.event if message is not None and hasattr(message, 'event') else message.data}")

        # If the message is not a device event, then don't do anything.
        if message.type_ != Cep2Zigbee2mqttMessageType.DEVICE_EVENT:
            return

        # Parse the topic to retreive the device ID. If the topic only has one level, don't do
        # anything.
        tokens = message.topic.split("/")
        if len(tokens) <= 1:
            return

        # Retrieve the device ID from the topic.
        device_id = tokens[1]
        # If the device ID is known, then process the device event and send a message to the remote
        # web server.
        device = self.__devices_model.find(device_id)

        if device:
            data = message.event
            mode = ""
            if "occupancy" in data.keys():
                if data["occupancy"] == True:
                    mode = "Occupied"
                else:
                    mode = "Not occupied"
            elif "state" in data.keys():
                mode = data["state"]
            else:
                raise Exception("Neither occupancy nor state keys found") 

            #print(type(device.type_))
            #print(type(device.id_))
            # Based on the value of occupancy, change the state of the actuators to ON
            # (occupancy is true, i.e. a person is present in the room) or OFF.
#            new_state = "ON" if occupancy else "OFF"
#            print("New state: ", new_state)
            # Change the state on all actuators, i.e. LEDs and power plugs.
 #           for a in self.__devices_model.actuators_list:
 #               self.__z2m_client.change_state(a.id_, new_state)
            daTime = f"{datetime.datetime.now()}"
            
            # Convert event_data to string
            data_str = f"{data}"
            
            # Register event in the remote web server.
            web_event = {
                "time_stamp": daTime, 
                "mode": mode,
                "event_data": data_str, 
                "event_type_enum": "17", 
                "patient_id": 5, 
                "device_model": device.type_, 
                "device_vendor": "Aqara", 
                "gateway_id": 13, 
                "id": device.id_
                }
            try:

    
                if device.type_ == "LED":
                    print("Trying to save: ", web_event)
                
                responseSave = requests.post(self.HTTP_HOST + "/api/save", json = web_event)
    
                print("Response from save function: ", responseSave)
            except ConnectionError as ex:
                print(f"{ex}")
