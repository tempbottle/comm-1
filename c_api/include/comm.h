#include <stdlib.h>

typedef void comm_address_t;
typedef void comm_udp_node_t;
typedef void comm_network_t;
typedef void comm_client_t;
typedef void comm_client_commands_t;
typedef void comm_text_message_t;
typedef void (*comm_event_received_text_message_callback)(comm_text_message_t *);

void comm_initialize();

/********************************************//**
 *  Address functions
 ***********************************************/

comm_address_t *comm_address_for_content(char *content);
comm_address_t *comm_address_from_str(char *str);
comm_address_t *comm_address_null();

/** Returns a copy of address without assuming ownership of it.
 *
 */
comm_address_t *comm_address_copy(comm_address_t *address);
char *comm_address_to_str(comm_address_t *address);
void comm_address_destroy(comm_address_t *address);

/********************************************//**
 *  UdpNode functions
 ***********************************************/

comm_udp_node_t *comm_udp_node_new(comm_address_t *address, char *socket_address);
void comm_udp_node_destroy(comm_udp_node_t *udp_node);

/********************************************//**
 *  Network functions
 ***********************************************/

/** Returns a new network instance
 *
 * @param self_node Ownership assumed.
 * @param host
 * @param routers array of router nodes. Ownership assumed.
 * @param routers_count number of router nodes in +routers+
 */
comm_network_t *comm_network_new(
        comm_udp_node_t *self_node,
        char *host,
        comm_udp_node_t **routers,
        size_t routers_count);
void comm_network_run(comm_network_t *);
void comm_network_destroy(comm_network_t *network);

/********************************************//**
 *  Message functions
 ***********************************************/

comm_text_message_t *comm_text_message_new(comm_address_t *sender, char *text);

/**
 * Returns a string with the text of a text message.
 */
char *comm_text_message_text(comm_text_message_t *text_message);

/**
 * Returns a borroed reference to +text_message+'s sender address.
 */
comm_address_t *comm_text_message_sender(comm_text_message_t *text_message);

/********************************************//**
 *  Client functions
 ***********************************************/

comm_client_t *comm_client_new(comm_address_t *address);

/**
 * Starts the client in a worker thread.
 *
 * @param client Ownership assumed.
 * @param network Ownership assumed.
 */
comm_client_commands_t *comm_client_run(
        comm_client_t *client, comm_network_t *network);

void comm_client_register_text_message_received_callback(
        comm_client_t *client, comm_event_received_text_message_callback callback);

/**
 * @param commands Reference to a client command sender. Reference is borrowed.
 * @param recipient Recipient address. Ownership assumed.
 * @param text_message Reference to a text message struct. Ownership assumed.
 */
void comm_client_commands_send_text_message(
        comm_client_commands_t *commands,
        comm_address_t *recipient,
        comm_text_message_t *text_message);

void comm_client_destroy(comm_client_t *client);
