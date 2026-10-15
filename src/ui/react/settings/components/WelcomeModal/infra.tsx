import { Button, Flex, Modal } from "antd";
import { z } from "zod";

import cs from "./index.module.css";
import { useTranslation } from "react-i18next";
import { persistentSignal } from "libs/ui/react/utils/PersistentSignal";

const WelcomeStateSchema = z.object({
  welcomeShown: z.boolean().default(false),
  alreadyReviewed: z.boolean().default(false),
  lastReviewPrompt: z.number().nullish(),
});

type WelcomeState = z.infer<typeof WelcomeStateSchema>;
const DEFAULT_STATE = WelcomeStateSchema.parse({});

const ReviewState = await persistentSignal("welcomeModal", DEFAULT_STATE, WelcomeStateSchema);

export const WelcomeModal = () => {
  const state = ReviewState.value;
  const isNewUser = !state.welcomeShown;
  const { t } = useTranslation();

  const markAsDone = () => {
    ReviewState.value = { ...ReviewState.value, alreadyReviewed: true, welcomeShown: true };
  };

  return (
    <>
      <Modal
        className={cs.welcome}
        open={isNewUser}
        centered
        closable={false}
        title={`🎉 ${t("welcome.title")} 🥳`}
        footer={
          <Flex justify="flex-end" gap="1rem">
            <Button
              type="primary"
              onClick={markAsDone}
            >
              {t("welcome.ok")}
            </Button>
          </Flex>
        }
      >
        <p>{t("welcome.message")}</p>
        <br />
        <p>{t("welcome.review")}</p>
        <br />
      </Modal>
    </>
  );
};
