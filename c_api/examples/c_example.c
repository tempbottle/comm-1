#include "comm.h"
#include <signal.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>

volatile sig_atomic_t stop;

void handle_sigint(int signum) {
    stop = 1;
}

void received_text_message_callback(comm_text_message_t *message) {
    comm_address_t *sender = comm_text_message_sender(message);
    printf("%s -> %s\n",
            comm_address_to_str(sender),
            comm_text_message_text(message));
}

int main(int argc, char *argv[]) {
    signal(SIGINT, handle_sigint);
    if (argc < 3) { return 1; }

    comm_initialize();

    char *secret = malloc(strlen(argv[1]) + 1);
    strcpy(secret, argv[1]);
    comm_address_t *address = comm_address_for_content(secret);

    comm_udp_node_t **routers;
    size_t routers_count = 0;
    if (argc > 3) {
        routers = malloc(sizeof(comm_udp_node_t *));
        char *router_socket_addr = malloc(strlen(argv[3]) + 1);
        strcpy(router_socket_addr, argv[3]);
        comm_udp_node_t *router = comm_udp_node_new(comm_address_null(), router_socket_addr);
        routers[0] = router;
        routers_count = 1;
    }

    char *host = malloc(strlen(argv[2]) + 1);
    strcpy(host, argv[2]);
    comm_network_t *network = comm_network_new(address, host, routers, routers_count);

    comm_address_t *client_address = comm_address_copy(address);
    comm_client_t *client = comm_client_new(client_address);
    comm_client_register_text_message_received_callback(client, received_text_message_callback);
    comm_client_commands_t *commands = comm_client_run(client, network);

    while (!stop) {
        char *line = NULL;
        size_t len;
        if (getline(&line, &len, stdin) == -1) {
            printf("No line\n");
            continue;
        } else if (strlen(line) < 43) {
            continue;
        }

        char recipient_str[41];
        strncpy(recipient_str, line, 40);
        recipient_str[40] = '\0';
        char *message_text = malloc(strlen(line) - 41); // addr and space
        strcpy(message_text, &line[41]);

        comm_address_t *recipient = comm_address_from_str(recipient_str);
        comm_address_t *sender = comm_address_copy(address);
        comm_text_message_t *text_message = comm_text_message_new(sender, message_text);
        comm_client_commands_send_text_message(commands, recipient, text_message);
        free(message_text);
    }

    free(host);
    free(secret);
}
