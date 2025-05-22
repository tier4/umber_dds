#include "dds/dds.h"
#include "Shape.h"
#include <stdio.h>
#include <stdlib.h>

static void on_offered_incompatible_qos(
    dds_entity_t entity,
    const dds_offered_incompatible_qos_status_t status,
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

static void on_publication_matched(
    dds_entity_t writer,
    const dds_publication_matched_status_t status,
    void *listener_data
)
{
    (void)writer;
    (void)listener_data;

    printf("[on_publication_matched]\n");
    printf("  total_count         : %d\n", status.total_count);
    printf("  total_count_change  : %d\n", status.total_count_change);
    printf("  current_count       : %d\n", status.current_count);
    printf("  current_count_change: %d\n", status.current_count_change);

}

static void on_liveliness_lost(
    dds_entity_t writer,
    const dds_liveliness_lost_status_t status,
    void *listener_data
)
{
    (void)writer;
    (void)listener_data;

    printf("[on_liveliness_lost]\n");
    printf("  total_count         : %d\n", status.total_count);
    printf("  total_count_change  : %d\n", status.total_count_change);

}

int main (int argc, char ** argv)
{
  dds_entity_t participant;
  dds_entity_t topic;
  dds_entity_t writer;
  dds_return_t rc;
  dds_qos_t *qos;
  ShapeType msg;
  dds_listener_t *listener = NULL;
  uint32_t status = 0;
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
  dds_lset_offered_incompatible_qos(listener, on_offered_incompatible_qos);
  dds_lset_publication_matched(listener, on_publication_matched);
  dds_lset_liveliness_lost(listener, on_liveliness_lost);
  // listener = NULL;

  /* Create a besteffort Writer. */
  qos = dds_create_qos ();
  dds_qset_reliability (qos, DDS_RELIABILITY_BEST_EFFORT, DDS_SECS (10));
  // dds_qset_liveliness (qos, DDS_LIVELINESS_MANUAL_BY_PARTICIPANT, DDS_SECS (5));
  writer = dds_create_writer (participant, topic, qos, listener);
  if (writer < 0)
    DDS_FATAL("dds_create_writer: %s\n", dds_strretcode(-writer));

  printf("=== [Publisher]  Waiting for a reader to be discovered ...\n");
  fflush (stdout);

  /*
  rc = dds_set_status_mask(writer, DDS_PUBLICATION_MATCHED_STATUS);
  if (rc != DDS_RETCODE_OK)
    DDS_FATAL("dds_set_status_mask: %s\n", dds_strretcode(-rc));

  while(!(status & DDS_PUBLICATION_MATCHED_STATUS))
  {
    rc = dds_get_status_changes (writer, &status);
    if (rc != DDS_RETCODE_OK)
      DDS_FATAL("dds_get_status_changes: %s\n", dds_strretcode(-rc));

    // Polling sleep.
    dds_sleepfor (DDS_MSECS (20));
  }
  */
  dds_publication_matched_status_t pm;
  do {
    dds_sleepfor (DDS_MSECS (20));
    rc = dds_get_publication_matched_status (writer, &pm);
    if (rc != DDS_RETCODE_OK)
        DDS_FATAL("dds_get_publication_matched_status: %s\n",
                  dds_strretcode (-rc));
  } while (pm.current_count == 0);

  /* Create a message to write. */
  msg.color = "RED";
  msg.x = 42;
  msg.y = 42;
  msg.shapesize = 42;


  while (1){
    printf ("=== [Publisher]  Writing : ");
    printf ("Message (%s, x:%"PRId32", y:%"PRId32", shapesize:%"PRId32")\n", msg.color, msg.x, msg.y, msg.shapesize);
    fflush (stdout);
    rc = dds_write (writer, &msg);
    msg.x += 5;
    msg.y += 5;
    msg.x = msg.x % 255;
    msg.y = msg.y % 255;
    sleep(1);
    if (rc != DDS_RETCODE_OK) {
      DDS_FATAL("dds_write: %s\n", dds_strretcode(-rc));
      break;
    }
    }

  /* Deleting the participant will delete all its children recursively as well. */
  rc = dds_delete (participant);
  if (rc != DDS_RETCODE_OK)
    DDS_FATAL("dds_delete: %s\n", dds_strretcode(-rc));

  return EXIT_SUCCESS;
}
