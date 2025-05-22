#include "dds/dds.h"
#include "Shape.h"
#include <stdio.h>
#include <string.h>
#include <stdlib.h>

/* An array of one message (aka sample in dds terms) will be used. */
#define MAX_SAMPLES 1

static void on_requested_incompatible_qos(
    dds_entity_t entity,
    const dds_requested_incompatible_qos_status_t status,
    void *listener_data
)
{
    (void)listener_data;

    printf("[on_offered_incompatible_qos]\n");
    printf("  entity           : %d\n", entity);
    printf("  last_policy_id   : %d\n", status.last_policy_id);
    printf("  total_count      : %u\n", status.total_count);
    printf("  total_count_change: %d\n", status.total_count_change);
}

static void on_subscription_matched(
    dds_entity_t reader,
    const dds_subscription_matched_status_t status,
    void *listener_data
)
{
    (void)reader;
    (void)listener_data;

    printf("[on_subscription_matched]\n");
    printf("  total_count         : %d\n", status.total_count);
    printf("  total_count_change  : %d\n", status.total_count_change);
    printf("  current_count       : %d\n", status.current_count);
    printf("  current_count_change: %d\n", status.current_count_change);

}

static void on_liveliness_changed(
    dds_entity_t reader,
    const dds_liveliness_changed_status_t status,
    void *listener_data
)
{
    (void)reader;
    (void)listener_data;

    printf("[on_liveliness_changed]\n");
    printf("  alive_count           : %d\n", status.alive_count);
    printf("  not_alive_count       : %d\n", status.not_alive_count);
    printf("  alive_count_change    : %d\n", status.alive_count_change);
    printf("  not_alive_count_change: %d\n", status.not_alive_count_change);

}

int main (int argc, char ** argv)
{
  dds_entity_t participant;
  dds_entity_t topic;
  dds_entity_t reader;
  ShapeType *msg;
  void *samples[MAX_SAMPLES];
  dds_sample_info_t infos[MAX_SAMPLES];
  dds_return_t rc;
  dds_qos_t *qos;
  dds_listener_t *listener = NULL;
  (void)argc;
  (void)argv;

  /* Create a Participant. */
  participant = dds_create_participant (DDS_DOMAIN_DEFAULT, NULL, NULL);
  if (participant < 0)
    DDS_FATAL("dds_create_participant: %s\n", dds_strretcode(-participant));

  /* Create a Topic. */
  topic = dds_create_topic (
    participant, &ShapeType_desc, "Square", NULL, NULL);
  if (topic < 0)
    DDS_FATAL("dds_create_topic: %s\n", dds_strretcode(-topic));

  /* Create listener register callback */
  listener = dds_create_listener(NULL);
  dds_lset_requested_incompatible_qos(listener, on_requested_incompatible_qos);
  dds_lset_subscription_matched(listener, on_subscription_matched);
  dds_lset_liveliness_changed(listener, on_liveliness_changed);
  // listener = NULL;

  /* Create a besteffort Reader. */
  qos = dds_create_qos ();
  dds_qset_reliability (qos, DDS_RELIABILITY_BEST_EFFORT, DDS_SECS (10));
  // dds_qset_liveliness (qos, DDS_LIVELINESS_MANUAL_BY_PARTICIPANT, DDS_SECS (7));
  reader = dds_create_reader (participant, topic, qos, listener);
  if (reader < 0)
    DDS_FATAL("dds_create_reader: %s\n", dds_strretcode(-reader));
  dds_delete_qos(qos);

  printf ("\n=== [Subscriber] Waiting for a sample ...\n");
  fflush (stdout);

  /* Initialize sample buffer, by pointing the void pointer within
   * the buffer array to a valid sample memory location. */
  samples[0] = ShapeType__alloc ();

  /* Poll until data has been read. */
  while (true)
  {
    /* Do the actual read.
     * The return value contains the number of read samples. */
    rc = dds_take (reader, samples, infos, MAX_SAMPLES, MAX_SAMPLES);
    if (rc < 0) {
      DDS_FATAL("dds_read: %s\n", dds_strretcode(-rc));
      break;
    }

    /* Check if we read some data and it is valid. */
    if ((rc > 0) && (infos[0].valid_data))
    {
      /* Print Message. */
      msg = (ShapeType*) samples[0];
      printf ("=== [Subscriber] Received : ");
      printf ("Message (%s, x:%"PRId32", y:%"PRId32", shapesize:%"PRId32")\n", msg->color, msg->x, msg->y, msg->shapesize);
      fflush (stdout);
      // break;
    }
    else
    {
      /* Polling sleep. */
      dds_sleepfor (DDS_MSECS (20));
    }

  }

  /* Free the data location. */
  ShapeType_free (samples[0], DDS_FREE_ALL);

  /* Deleting the participant will delete all its children recursively as well. */
  rc = dds_delete (participant);
  if (rc != DDS_RETCODE_OK)
    DDS_FATAL("dds_delete: %s\n", dds_strretcode(-rc));

  return EXIT_SUCCESS;
}
