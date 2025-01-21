pub const NAME_VEC: [&str; 18] = [
    INTENT_INPUT,
    MYSQL, MONGODB, GOOGLE_DRIVE, SMART_DISHWASHER, 
    SMART_ITEM_SORTING_SYSTEM, SELF_DRIVING_CAR,
    SMART_PHONE, CONFERENCE_ROOM_MANAGEMENT_SYSTEM,
    MAIL_SYSTEM, IT_SUPPORT_TEAM, DELIVERY_SERVICE,
    SMART_LIGHTING_SYSTEM, INSTANT_WATER_HEATER,
    SMART_SWEEPING_ROBOT, SMART_REFRIGERATOR,
    SMART_SPEAKER, SMART_DISINFECTION_CABINET
];

pub const DESCRIPTION_VEC: [&str; 18] = [
    INTENT_INPUT_DESCRIPTION,
    MYSQL_DESCRIPTION, MONGODB_DESCRIPTION, GOOGLE_DRIVE_DESCRIPTION, SMART_DISHWASHER_DESCRIPTION, 
    SMART_ITEM_SORTING_SYSTEM_DESCRIPTION, SELF_DRIVING_CAR_DESCRIPTION,
    SMART_PHONE_DESCRIPTION, CONFERENCE_ROOM_MANAGEMENT_SYSTEM_DESCRIPTION,
    MAIL_SYSTEM_DESCRIPTION, IT_SUPPORT_TEAM_DESCRIPTION, DELIVERY_SERVICE_DESCRIPTION,
    SMART_LIGHTING_SYSTEM_DESCRIPTION, INSTANT_WATER_HEATER_DESCRIPTION,
    SMART_SWEEPING_ROBOT_DESCRIPTION, SMART_REFRIGERATOR_DESCRIPTION,
    SMART_SPEAKER_DESCRIPTION, SMART_DISINFECTION_CABINET_DESCRIPTION
];

pub const MYSQL: &str = "MySQL";
pub const MYSQL_DESCRIPTION: &str = "MySQL can store, organize, and manage data in structured tables. It allows users to create, read, update, and delete data using SQL queries. It supports data sorting, filtering, and searching, and can handle complex operations like joining multiple tables. MySQL ensures data integrity through constraints, transactions, and indexing. It can manage large datasets, support multiple users simultaneously, and provide secure access control. Additionally, it enables backups, replication, and scalability for growing applications.";
pub const MONGODB: &str = "MongoDB";
pub const MONGODB_DESCRIPTION: &str = "MongoDB is a NoSQL database that stores data in flexible, JSON-like documents instead of tables. It can handle unstructured or semi-structured data, making it ideal for dynamic or evolving data models. MongoDB allows you to store, query, and manage large volumes of data efficiently. It supports indexing for fast searches, horizontal scaling for handling big data, and replication for high availability. MongoDB also enables complex queries, aggregation, and real-time analytics, making it suitable for modern applications with diverse data needs.";
pub const GOOGLE_DRIVE: &str = "Google Drive";
pub const GOOGLE_DRIVE_DESCRIPTION: &str = "Google Drive is a cloud-based storage service that allows you to store, share, and access files from anywhere. It can store documents, photos, videos, and other file types, and sync them across devices. You can create and edit files using Google Workspace tools like Docs, Sheets, and Slides directly within Drive. It supports file sharing with customizable permissions, collaboration in real-time, and version history to track changes. Google Drive also provides search functionality to quickly find files and integrates with other Google services and third-party apps.";
pub const SMART_PHONE: &str = "Smartphone";
pub const SMART_PHONE_DESCRIPTION: &str = "A smartphone is a multifunctional portable device that has real-time communication, navigation, transportation service scheduling, itinerary booking, information recording, schedule management, application operation, timer setting, list management and other functions. Users can use it to obtain real-time location information, plan routes, book transportation (such as flights, taxis, etc.), receive and send meeting notices, view meeting schedules, record temporary ideas, find information, set reminders, record task lists, and synchronize data with other devices.";
pub const SMART_ITEM_SORTING_SYSTEM: &str = "Smart home system";
pub const SMART_ITEM_SORTING_SYSTEM_DESCRIPTION: &str = "The smart item sorting system is an automated management solution based on preset conditions, which can assist in sorting and packing specific items (such as luggage, documents, etc.) according to user instructions or preset rules. It optimizes the item sorting process by linking with the user's other devices (such as smartphones, smart wardrobes, etc.) to ensure that the required items are complete and in order.";
pub const SELF_DRIVING_CAR: &str = "Self-driving car";
pub const SELF_DRIVING_CAR_DESCRIPTION: &str = "Self-driving car is a means of transportation with autonomous navigation capabilities. It can automatically plan routes and arrive at designated locations safely based on the destination information entered by the user. It is suitable for short or long-distance travel and can be seamlessly connected with other transportation services (such as flights);";
pub const CONFERENCE_ROOM_MANAGEMENT_SYSTEM: &str = "Conference room management system";
pub const CONFERENCE_ROOM_MANAGEMENT_SYSTEM_DESCRIPTION: &str = "The conference room management system is an integrated resource management tool with functions such as conference room reservation, equipment management, schedule synchronization and resource allocation. Users can use it to view the available time of the conference room, reserve or cancel the conference room, remotely control the equipment in the conference room (such as projectors, microphones, speakers, etc.), and automatically synchronize the meeting schedule to the calendar of the participants to ensure the efficient use of conference resources and the smooth execution of the meeting arrangement;";
pub const MAIL_SYSTEM: &str = "Mail system";
pub const MAIL_SYSTEM_DESCRIPTION: &str = "The mail system is an email-based communication and collaboration tool with functions such as notification sending, schedule management, file sharing and feedback collection. Users can use it to send meeting notices in batches, share meeting materials and agendas, collect feedback or confirmation information from participants, and synchronize the meeting schedule with the conference room management system to ensure the timeliness and accuracy of information transmission;";
pub const IT_SUPPORT_TEAM: &str = "IT support team";
pub const IT_SUPPORT_TEAM_DESCRIPTION: &str = "The IT support team is a group of professional technical support personnel with functions such as equipment debugging, troubleshooting, system maintenance and user training. Users can use them to debug conference room equipment (such as projectors, microphones, etc.) before the meeting, provide real-time technical support during the meeting to solve equipment or network problems, regularly maintain the conference room management system and related equipment to ensure its long-term stable operation, and provide equipment use training for participants to ensure the normal operation and efficient use of conference equipment;";
pub const DELIVERY_SERVICE:&str = "Delivery service";
pub const DELIVERY_SERVICE_DESCRIPTION:&str = "Delivery service is a delivery solution based on an online platform that supports users to order items according to their needs and deliver them to designated locations. Users can use it to quickly obtain the items they need and ensure the timely supply of items. It is suitable for item procurement and delivery needs in various scenarios.";
pub const SMART_LIGHTING_SYSTEM:&str = "Smart lighting system";
pub const SMART_LIGHTING_SYSTEM_DESCRIPTION:&str = "The smart lighting system is an adjustable lighting solution that supports adjusting the light brightness and color temperature according to the scene requirements. Users can use it to create a lighting environment suitable for different activities, which is suitable for lighting management in home, office or commercial scenarios.";
pub const INSTANT_WATER_HEATER:&str = "Instant water heater";
pub const INSTANT_WATER_HEATER_DESCRIPTION:&str = "An instant water heater is a fast heating device that supports instant hot water. Users can use it to quickly obtain hot water, which is suitable for hot water needs in various scenarios, such as cleaning, washing or drinking.";
pub const SMART_DISHWASHER:&str = "Smart dishwasher";
pub const SMART_DISHWASHER_DESCRIPTION:&str = "A smart dishwasher is an automated cleaning device that supports washing tableware and adjusts the washing mode according to the degree of stains. Users can use it to efficiently complete cleaning tasks, which is suitable for tableware cleaning needs in home or commercial scenarios.";
pub const SMART_SWEEPING_ROBOT:&str = "Smart sweeping robot";
pub const SMART_SWEEPING_ROBOT_DESCRIPTION:&str = "A smart sweeping robot is an automated cleaning device that supports floor cleaning and garbage collection. Users can use it to keep the floor clean, which is suitable for floor cleaning needs in home, office or commercial scenarios.";
pub const SMART_DISINFECTION_CABINET:&str = "Smart disinfection cabinet";
pub const SMART_DISINFECTION_CABINET_DESCRIPTION:&str = "A smart disinfection cabinet is a device with sterilization and storage functions, which supports efficient disinfection and classified storage of items. Users can use it to ensure the hygiene and safety of items, which is suitable for the disinfection and storage needs of items in home, medical or commercial scenarios.";
pub const SMART_SPEAKER:&str = "Smart speaker";
pub const SMART_SPEAKER_DESCRIPTION:&str = "A smart speaker is a voice interaction device that supports voice control to play media content, query information, set reminders and other functions. Users can use it to obtain real-time information, play music or audio content, and interact with other smart devices in a basic way. It is suitable for convenient information acquisition and entertainment experience in various scenarios.";
pub const SMART_REFRIGERATOR:&str = "Smart refrigerator";
pub const SMART_REFRIGERATOR_DESCRIPTION:&str = "A smart refrigerator is a smart home appliance with storage management and status monitoring functions. It supports core functions such as refrigeration, freezing, and preservation. At the same time, it can check the status of stored items, remind to replenish items, optimize energy consumption, etc. Users can use it to understand the quantity and status of stored items in real time, and generate a replenishment list as needed. It is suitable for item storage and management in home or commercial scenarios.";


pub const INTENT_INPUT: &str = "Intent Input";
pub const INTENT_INPUT_DESCRIPTION: &str = "Intent Input is a device which can get intent from user, but can not reveive any intent from other ways";
