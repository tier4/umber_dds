#include "dds/dds.h"
#include "Shape.h"
#include <stdio.h>
#include <stdlib.h>
#include <time.h>

int main (int argc, char ** argv)
{
  dds_entity_t participant;
  dds_entity_t topic;
  dds_entity_t writer;
  dds_return_t rc;
  dds_qos_t *qos;
  ShapeType msg;
  uint32_t status = 0;
  time_t start_time;
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

  /* Create a besteffort Writer. */
  qos = dds_create_qos ();
  dds_qset_reliability (qos, DDS_RELIABILITY_BEST_EFFORT, DDS_SECS (10));
  writer = dds_create_writer (participant, topic, qos, NULL);
  if (writer < 0)
    DDS_FATAL("dds_create_writer: %s\n", dds_strretcode(-writer));

  printf("=== [Publisher]  Waiting for a reader to be discovered ...\n");
  fflush (stdout);

  rc = dds_set_status_mask(writer, DDS_PUBLICATION_MATCHED_STATUS);
  if (rc != DDS_RETCODE_OK)
    DDS_FATAL("dds_set_status_mask: %s\n", dds_strretcode(-rc));

  while(!(status & DDS_PUBLICATION_MATCHED_STATUS))
  {
    rc = dds_get_status_changes (writer, &status);
    if (rc != DDS_RETCODE_OK)
      DDS_FATAL("dds_get_status_changes: %s\n", dds_strretcode(-rc));

    /* Polling sleep. */
    dds_sleepfor (DDS_MSECS (20));
  }

  /* Create a message to write. */
  msg.color = "RED";
  msg.x = 42;
  msg.y = 42;
  msg.shapesize = 42;

  start_time = time(NULL);
  if (start_time == (time_t)-1) {
    printf ("get time failed");
    return -1;
  }

  while (true){
    if (difftime(time(NULL), start_time) > 30.0) {
      break;
    }
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
