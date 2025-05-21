from Worker import (Cep2Zigbee2mqttClient,
                                   Cep2Zigbee2mqttMessage, Cep2Zigbee2mqttMessageType)
import requests
import datetime
import environment

class MQTT_Listener:
    """ The controller is responsible for managing events received from zigbee2mqtt and handle them.
    By handle them it can be process, store and communicate with other parts of the system. In this
    case, the class listens for zigbee2mqtt events, processes them (turn on another Zigbee device)
    and send an event to a remote HTTP server.
    """

    def __init__(self) -> None:
        """ Class initializer. The actuator and monitor devices are loaded (filtered) only when the
        class is instantiated. If the database changes, this is not reflected.

        Args:
            devices_model (Cep2Model): the model that represents the data of this application
        """
        self.__z2m_client = Cep2Zigbee2mqttClient(host=environment.MQTT_BROKER_HOST,
                                                  port=environment.MQTT_BROKER_PORT,
                                                  on_message_clbk=self.__zigbee2mqtt_event_received)
        
        self.old_web_event_power_plug = 0
        
    def start(self) -> None:
        """ Start listening for zigbee2mqtt events.
        """
        self.__z2m_client.connect()
        # A new event is sent as an HTTP GET request
        response = requests.get(environment.HTTP_HOST + "/api/status")
        if response.status_code == 200:
            print("Status of database was a SUCCESS!")
        else:
            print("Response from request: ", response)        

    def __zigbee2mqtt_event_received(self, message: Cep2Zigbee2mqttMessage) -> None:
        """ Process an event received from zigbee2mqtt. """
        if not message:
            print("NO MESSAGE RECIEVED")
            return

        # Blacklist this topic
        if message.topic != environment.ZIGBEE_METADATA_TOPIC and message.topic != environment.INTERVIEW_RESPONSE_TOPIC and message.topic != environment.INTERVIEW_REQUEST_TOPIC:
            print(f"zigbee2mqtt event received on topic {message.topic}: {message.event if hasattr(message, 'event') else message.data}")
        
        # If the message is not a device event, then don't do anything.
        if message.type_ != Cep2Zigbee2mqttMessageType.DEVICE_EVENT:
            return

        # Parse the topic to retreive the device ID. If the topic only has one level, don't do
        # anything.
        tokens = message.topic.split("/")
        if len(tokens) <= 1:
            return

        # Retrieve the device ID from the topic.
        device = tokens[1]
        # If the device ID is known, then process the device event and send a message to the remote
        # web server.
        if not device or device == "bridge":
            return

        data = message.event
        mode = ""
        # now we check 
        if "occupancy" in data.keys():
            mode = data["occupancy"]
        elif "power" in data.keys() and int(data["power"]) != self.old_web_event_power_plug:
            current_power = int(data["power"])
            self.old_web_event_power_plug = current_power
            if (current_power > 0):
                mode = "ON"
            else : 
                mode = "OFF"
        else:
            return

        crnt_time = f"{datetime.datetime.now()}"
        # Convert event_data to string
        data_str = f"{data}"
        # Register event in the remote web server.
        web_event = {
            "time_stamp": crnt_time, 
            "mode": str(mode),
            "event_data": data_str, 
            "event_type_enum": "17",
            "res_id": environment.RES_ID, 
            "device_model": device, 
            "device_vendor": "Aqara", 
            "gateway_id": environment.GATEWAY_ID, 
            "id": next((k for k, v in environment.SENSOR_DICT.items() if v == device), None)
            }

        try:
            print("Trying to save: ", web_event)
            responseSave = requests.post(environment.DB_ENDPOINT, json = web_event)
            print("Response from save function: ", responseSave)
                
        except ConnectionError as ex:
            print(f"{ex}")
